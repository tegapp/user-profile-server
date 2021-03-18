use eyre::{
    eyre,
    Result,
    Context as _,
};
use serde::Serialize;
use async_graphql::{Context, FieldResult, ID};
use host_connector::{HostConnection, HostConnectionResponse, Signal};
use prost::Message;

use crate::host_connector;
use crate::host::Host;
use crate::protos::InviteCode;

#[derive(async_graphql::InputObject, Debug)]
pub struct RegisterMachinesInput {
    pub machines: Vec<MachineInput>,
}

#[derive(async_graphql::InputObject, Debug)]
pub struct DeleteMachinesInput {
    pub machine_slugs: Vec<String>,
}

#[derive(async_graphql::InputObject, Debug)]
pub struct MachineInput {
    slug: String,
    name: String,
}

#[derive(async_graphql::InputObject, Debug)]
pub struct RespondToConnectionRequestInput {
    #[graphql(name = "sessionID")]
    pub session_id: ID,
    pub answer: SessionDescriptionInput,
    pub ice_candidates: Vec<async_graphql::Json<serde_json::Value>>,
}

#[derive(async_graphql::InputObject, Serialize, Debug)]
pub struct SessionDescriptionInput {
    pub sdp: async_graphql::Json<serde_json::Value>,
    pub r#type: async_graphql::Json<serde_json::Value>,
}

#[derive(async_graphql::InputObject, Debug)]
#[graphql(name = "SendICECandidatesInput")]
pub struct SendIceCandidatesInput {
    #[graphql(name = "sessionID")]
    pub session_id: ID,
    pub ice_candidates: Vec<async_graphql::Json<serde_json::Value>>,
}

#[derive(async_graphql::InputObject, Debug)]
pub struct ConnectToHostInput {
    pub host_slug: Option<String>,
    pub invite: Option<String>,
    pub offer: async_graphql::Json<serde_json::Value>,
}

#[derive(Default, Clone, Copy)]
pub struct HostMutation;

#[async_graphql::Object]
impl HostMutation {
    #[instrument(skip(self, ctx))]
    async fn register_machines_from_host<'ctx>(
        &self,
        ctx: &'ctx Context<'_>,
        input: RegisterMachinesInput,
    ) -> FieldResult<Option<crate::Void>> {
        let db: &crate::Db = ctx.data()?;
        let auth: &crate::AuthContext = ctx.data()?;

        async move {
            let host = auth.require_host()?;

            for m in input.machines.into_iter() {
                sqlx::query!(
                    r#"
                        INSERT INTO machines (host_id, name, slug)
                        VALUES ($1, $2, $3)
                        ON CONFLICT (host_id, slug)
                        DO UPDATE SET
                            name=$2
                        RETURNING *
                    "#,
                    host.id,
                    m.name,
                    m.slug,
                )
                    .fetch_one(db)
                    .await?;
            }

            eyre::Result::<_>::Ok(None)
        }
            // log the backtrace which is otherwise lost by FieldResult
            .await
            .map_err(|err| {
                warn!("{:?}", err);
                err.into()
            })
    }

    async fn delete_machines_from_host<'ctx>(
        &self,
        ctx: &'ctx Context<'_>,
        input: DeleteMachinesInput,
    ) -> FieldResult<Option<crate::Void>> {
        let db: &crate::Db = ctx.data()?;
        let auth: &crate::AuthContext = ctx.data()?;

        async move {
            let host = auth.require_host()?;

            sqlx::query!(
                r#"
                    DELETE FROM machines
                    WHERE
                        host_id=$1
                        AND slug = ANY($2)
                "#,
                host.id,
                &input.machine_slugs[..],
            )
                .fetch_optional(db)
                .await?;

            eyre::Result::<_>::Ok(None)
        }
            // log the backtrace which is otherwise lost by FieldResult
            .await
            .map_err(|err| {
                warn!("{:?}", err);
                err.into()
            })
    }

    #[instrument(skip(self, ctx))]
    async fn connect_to_host<'ctx>(
        &self,
        ctx: &'ctx Context<'_>,
        input: ConnectToHostInput,
    ) -> FieldResult<HostConnection> {
        let db: &crate::Db = ctx.data()?;
        let auth: &crate::AuthContext = ctx.data()?;
        let host_connectors: &crate::HostConnectorsMap = ctx.data()?;

        let ice_servers: &crate::IceServerList = ctx.data()?;
        let ice_servers = (**ice_servers.load()).clone();

        async move {
            let user = auth.require_authorized_user()?;

            // Parse the invite
            let invite = input.invite
                .as_ref()
                .map(|invite| -> Result<_> {
                    let invite = bs58::decode(invite).into_vec()?;

                    let invite: InviteCode = Message::decode(&invite[..])?;

                    Ok(invite)
                })
                .transpose()?;

            let host_slug = if let Some(host_slug) = input.host_slug {
                host_slug
            } else if let Some(invite) = invite {
                // Encode the public key in base58 to get the slug
                bs58::encode(invite.host_public_key).into_string()
            } else {
                Err(eyre!("A hostSlug or invite is required"))?
            };

            let host = sqlx::query_as!(
                Host,
                r#"
                    SELECT * FROM hosts WHERE slug = $1
                "#,
                host_slug,
            )
                .fetch_one(db)
                .await
                .wrap_err_with(||
                    r#"
                    Printer appears to be offline.
                    Make sure it is plugged in and connected to wifi.
                    "#
                )?;

            let connector = host_connectors.get(&host.id)
                .and_then(|weak_addr| weak_addr.upgrade())
                .ok_or_else(|| eyre!(
                    r#"
                    Printer appears to be offline.
                    Make sure it is plugged in and connected to wifi.
                    "#
                ))?;

            let session_id: ID = nanoid!().into();

            connector.call(Signal {
                user_id: user.id.into(),
                email: Some(user.email.clone()),
                email_verified: user.email_verified,
                invite: input.invite,
                session_id: session_id.clone(),
                offer: input.offer,
                ice_servers,
            }).await??;

            Result::<_>::Ok(HostConnection {
                host,
                session_id,
            })
        }
            // log the backtrace which is otherwise lost by FieldResult
            .await
            .map_err(|err| {
                warn!("{:?}", err);
                err.into()
            })
    }

    /// After a connection has been received by the host (via `connectionRequested`) the
    /// host MAY choose to respond to the client via `respondToConnectionRequest`.
    #[instrument(skip(self, ctx))]
    async fn respond_to_connection_request<'ctx>(
        &self,
        ctx: &'ctx Context<'_>,
        input: RespondToConnectionRequestInput,
    ) -> FieldResult<Option<crate::Void>> {
        let auth: &crate::AuthContext = ctx.data()?;

        let host = auth.require_host()?;

        let RespondToConnectionRequestInput {
            session_id,
            answer,
            ice_candidates,
        } = input;

        let response_senders: &crate::ConnectionResponseSenders = ctx.data()?;

        let key = (host.id, session_id.clone());

        let sender = if let
            Some((_, sender)) = response_senders.remove(&key)
        {
            sender
        } else {
            debug!("Orphined session ({:?})", session_id);
            return Ok(None)
        };

        let answer = async_graphql::Json::from(serde_json::to_value(answer)?);

        if let Err(_) = sender.send(HostConnectionResponse {
            answer,
            ice_candidates,
        }) {
            debug!("Orphined session ({:?})", session_id);
            return Ok(None)
        }

        Ok(None)
    }

    // #[graphql(name = "sendICECandidatesToClient")]
    // async fn send_ice_candidates_to_client<'ctx>(
    //     &self,
    //     ctx: &'ctx Context<'_>,
    //     input: SendIceCandidatesInput,
    // ) -> FieldResult<Option<crate::Void>> {
    //     let db: &crate::Db = ctx.data()?;
    //     let auth: &crate::AuthContext = ctx.data()?;

    //     let host = auth.require_host()?;

    //     // TODO

    //     Ok(None)
    // }

    async fn remove_host_from_user<'ctx>(
        &self,
        ctx: &'ctx Context<'_>,
        machine_id: ID
    ) -> FieldResult<Option<crate::Void>> {
        let db: &crate::Db = ctx.data()?;
        let auth: &crate::AuthContext = ctx.data()?;

        let user = auth.require_authorized_user()?;

        let machine_id = machine_id.parse::<i64>().wrap_err( "Invalid machine id")?;

        sqlx::query!(
            "
                DELETE FROM hosts_users WHERE user_id = $1 AND id = $2
            ",
            user.id,
            machine_id,
        )
            .execute(db)
            .await
            .wrap_err( "Unable to delete machine")?;

        Ok(None)
    }
}
