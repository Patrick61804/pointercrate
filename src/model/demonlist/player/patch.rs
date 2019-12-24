use super::{DatabasePlayer, FullPlayer};
use crate::{
    citext::CiString,
    context::RequestContext,
    error::PointercrateError,
    model::{demonlist::player::Player, nationality::Nationality},
    operation::{deserialize_non_optional, deserialize_optional, Get, Patch},
    schema::players,
    Result,
};
use diesel::{result::Error, Connection, ExpressionMethods, RunQueryDsl};
use log::info;
use serde_derive::Deserialize;

make_patch! {
    struct PatchPlayer {
        name: CiString,
        banned: bool,
        nationality: Option<String>,
    }
}

impl Patch<PatchPlayer> for Player {
    fn patch(mut self, patch: PatchPlayer, ctx: RequestContext) -> Result<Self> {
        ctx.check_permissions(perms!(ListModerator or ListAdministrator))?;
        ctx.check_if_match(&self)?;

        let connection = ctx.connection();

        info!("Patching player {} with {}", self, patch);

        connection.transaction(|| {
            if let Some(true) = patch.banned {
                if !self.banned {
                    self.ban(connection)?;
                }
            }

            if let Some(ref name) = patch.name {
                if *name != self.name {
                    match DatabasePlayer::by_name(name.as_ref()).first(connection) {
                        Ok(player) => self.merge(player, connection)?,
                        Err(Error::NotFound) => (),
                        Err(err) => return Err(PointercrateError::database(err)),
                    }
                }
            }

            if let Some(nationality) = patch.nationality {
                self.nationality = nationality
                    .map(|nation| Nationality::get(nation.as_ref(), ctx))
                    .transpose()?;
            }

            patch!(self, patch: name, banned);

            diesel::update(players::table)
                .filter(players::id.eq(&self.id))
                .set((
                    players::banned.eq(&self.banned),
                    players::name.eq(&self.name),
                    players::nationality
                        .eq(&self.nationality.as_ref().map(|n| &n.iso_country_code)),
                ))
                .execute(connection)?;

            Ok(self)
        })
    }
}

impl Patch<PatchPlayer> for FullPlayer {
    fn patch(self, patch: PatchPlayer, ctx: RequestContext) -> Result<Self> {
        Ok(FullPlayer {
            player: self.player.patch(patch, ctx)?,
            ..self
        })
    }
}
