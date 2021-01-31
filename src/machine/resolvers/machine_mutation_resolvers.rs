use eyre::{
    eyre,
    Result,
    Context as _,
};
use serde::Serialize;
use async_graphql::{Context, FieldResult, ID};
use host_connector::{HostConnection, HostConnectionResponse, Signal};

use crate::host_connector;
use crate::host::Host;

#[derive(async_graphql::InputObject, Debug)]
pub struct RegisterMachinesInput {
    pub machines: Vec<MachineInput>,
}

#[derive(async_graphql::InputObject, Debug)]
pub struct MachineInput {
    slug: String,
    name: String,
}

#[derive(async_graphql::InputObject, Debug)]
pub struct AnswerSignalInput {
    #[graphql(name = "sessionID")]
    pub session_id: ID,
    pub answer: SessionDescriptionInput,
    pub ice_candidates: Vec<IceCandidateInput>,
}

#[derive(async_graphql::InputObject, Serialize, Debug)]
pub struct SessionDescriptionInput {
    pub sdp: async_graphql::Json<serde_json::Value>,
    pub desc_type: async_graphql::Json<serde_json::Value>,
}


#[derive(async_graphql::InputObject, Serialize, Debug)]
pub struct IceCandidateInput {
    pub candidate: String,
    pub mid: String,
}

#[derive(async_graphql::InputObject, Debug)]
#[graphql(name = "SendICECandidatesInput")]
pub struct SendIceCandidatesInput {
    #[graphql(name = "sessionID")]
    pub session_id: ID,
    pub ice_candidates: IceCandidateInput,
}

#[derive(async_graphql::InputObject, Debug)]
pub struct ConnectToHostInput {
    pub host_slug: String,
    pub offer: async_graphql::Json<serde_json::Value>,
}

pub struct Mutation;

#[async_graphql::Object]
impl Mutation {
    #[instrument(skip(self, ctx))]
    async fn register_machines<'ctx>(
        &self,
        ctx: &'ctx Context<'_>,
        input: RegisterMachinesInput,
    ) -> FieldResult<Option<crate::Void>> {
        let db: &crate::Db = ctx.data()?;
        let auth: &crate::AuthContext = ctx.data()?;

        let host = auth.require_host()?;

        for m in input.machines.into_iter() {
            sqlx::query!(
                r#"
                    INSERT INTO machines (host_id, name, slug)
                    VALUES ($1, $2, $3)
                    ON CONFLICT (host_id, slug)
                    DO UPDATE SET
                        name=$2
                "#,
                host.id,
                m.name,
                m.slug,
            )
                .fetch_one(db)
                .await?;
        }

        Ok(None)
    }

    #[instrument(skip(self, ctx))]
    async fn connect_to_host<'ctx>(
        &self,
        ctx: &'ctx Context<'_>,
        input: ConnectToHostInput,
    ) -> FieldResult<HostConnection> {
        let db: &crate::Db = ctx.data()?;
        let host_connectors: &crate::HostConnectorsMap = ctx.data()?;

        let host = sqlx::query_as!(
            Host,
            r#"
                SELECT * FROM hosts WHERE slug = $1
            "#,
            input.host_slug,
        )
            .fetch_one(db)
            .await?;

        let connector = host_connectors.get(&host.id)
            .and_then(|weak_addr| weak_addr.upgrade())
            .ok_or_else(|| eyre!(r#"
                Printer host appears to be offline. \
                Make sure it is plugged in and connected to wifi.
            "#))?;

        let session_id: ID = nanoid!().into();

        connector.call(Signal {
            session_id: session_id.clone(),
            offer: input.offer,
        }).await??;

        Ok(HostConnection {
            host,
            session_id,
        })
    }

    #[instrument(skip(self, ctx))]
    async fn answer_signal<'ctx>(
        &self,
        ctx: &'ctx Context<'_>,
        input: AnswerSignalInput,
    ) -> FieldResult<Option<crate::Void>> {
        let auth: &crate::AuthContext = ctx.data()?;

        let host = auth.require_host()?;

        let AnswerSignalInput {
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

        let ice_candidates = ice_candidates
            .into_iter()
            .map(|ic| Ok(async_graphql::Json::from(serde_json::to_value(ic)?)))
            .collect::<Result<Vec<async_graphql::Json<serde_json::Value>>>>()?;

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

    async fn remove_user_from_host<'ctx>(
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
