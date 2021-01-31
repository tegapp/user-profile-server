use eyre::{
    // eyre,
    // Result,
    Context as _,
};
use async_graphql::{Context, FieldResult, ID};

#[derive(async_graphql::InputObject)]
pub struct RegisterMachinesInput {
    pub machines: MachineInput,
}

#[derive(async_graphql::InputObject)]
pub struct MachineInput {
    id: ID,
    name: String,
}

#[derive(async_graphql::InputObject)]
pub struct AnswerSignalInput {
    #[graphql(name = "sessionID")]
    pub session_id: ID,
    pub answer: SessionDescriptionInput,
    pub ice_candidates: IceCandidateInput,
}

#[derive(async_graphql::InputObject)]
pub struct SessionDescriptionInput {
    pub sdp: async_graphql::Json<serde_json::Value>,
    pub desc_type: async_graphql::Json<serde_json::Value>,
}

#[derive(async_graphql::InputObject)]
pub struct IceCandidateInput {
    pub candidate: String,
    pub mid: String,
}


#[derive(async_graphql::InputObject)]
#[graphql(name = "SendICECandidatesInput")]
pub struct SendIceCandidatesInput {
    #[graphql(name = "sessionID")]
    pub session_id: ID,
    pub ice_candidates: IceCandidateInput,
}

pub struct Mutation;

#[async_graphql::Object]
impl Mutation {
    async fn register_machines<'ctx>(
        &self,
        ctx: &'ctx Context<'_>,
        input: RegisterMachinesInput,
    ) -> FieldResult<Option<crate::Void>> {
        let db: &crate::Db = ctx.data()?;
        let auth: &crate::AuthContext = ctx.data()?;

        let host = auth.require_host()?;

        // TODO

        Ok(None)
    }

    async fn answer_signal<'ctx>(
        &self,
        ctx: &'ctx Context<'_>,
        input: RegisterMachinesInput,
    ) -> FieldResult<Option<crate::Void>> {
        let db: &crate::Db = ctx.data()?;
        let auth: &crate::AuthContext = ctx.data()?;

        let host = auth.require_host()?;

        // TODO

        Ok(None)
    }

    #[graphql(name = "sendICECandidates")]
    async fn send_ice_candidates<'ctx>(
        &self,
        ctx: &'ctx Context<'_>,
        input: SendIceCandidatesInput,
    ) -> FieldResult<Option<crate::Void>> {
        let db: &crate::Db = ctx.data()?;
        let auth: &crate::AuthContext = ctx.data()?;

        let host = auth.require_host()?;

        // TODO

        Ok(None)
    }

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
