use crate::{
    utils::{format_user_ids_list, user_ids_below_user, user_ids_from},
    Context, Error,
};

use poise::CreateReply;

use serenity::{
    all::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, GuildId, Http, Timestamp, UserId},
    futures::future::join_all,
};

#[poise::command(
    slash_command,
    prefix_command,
    required_bot_permissions = "MANAGE_MESSAGES",
    required_permissions = "MANAGE_MESSAGES",
    category = "Moderation",
    guild_only
)]
pub async fn kick(ctx: Context<'_>, users: String, reason: Option<String>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let reason = reason.as_deref();
    let users = user_ids_from(&users);
    let users = user_ids_below_user(&ctx, users, ctx.author().id);
    let guild_id = ctx.guild_id().unwrap();
    let http = ctx.http();
    let results = kick_members(http, guild_id, users, reason).await?;

    let punished_users = results
        .into_iter()
        .filter_map(|r| r.ok())
        .collect::<Vec<UserId>>();

    let embed = build_kick_embed(&ctx, punished_users, reason.unwrap_or("No reason provided"));
    let builder = CreateReply::default().embed(embed).ephemeral(true);
    ctx.send(builder).await?;
    Ok(())
}

fn build_kick_embed(ctx: &Context<'_>, user_ids: Vec<UserId>, reason: &str) -> CreateEmbed {
    let author = ctx.author();
    let client = ctx.cache().current_user();
    let author_avatar_url = author.avatar_url().unwrap_or(author.default_avatar_url());
    let client_avatar_url = client.avatar_url().unwrap_or(client.default_avatar_url());
    let embed_author = CreateEmbedAuthor::new(&author.name).icon_url(author_avatar_url);
    let embed_footer = CreateEmbedFooter::new(&client.name).icon_url(client_avatar_url);
    let timestamp = Timestamp::now();
    let amount = user_ids.len();
    let description = format!("**{amount}** users kicked out!");

    CreateEmbed::new()
        .author(embed_author)
        .footer(embed_footer)
        .timestamp(timestamp)
        .color(0x3BA55C)
        .title("Kick")
        .description(description)
        .field("Reason", reason, false)
        .field("Users", format_user_ids_list(user_ids), false)
}

async fn kick_members(
    http: impl AsRef<Http>,
    guild_id: GuildId,
    user_ids: impl IntoIterator<Item = UserId>,
    reason: Option<&str>,
) -> Result<Vec<Result<UserId, Error>>, Error> {
    let futures = user_ids
        .into_iter()
        .map(|user_id| kick_member(&http, guild_id, user_id, reason))
        .collect::<Vec<_>>();

    Ok(join_all(futures).await)
}

async fn kick_member(
    http: impl AsRef<Http>,
    guild_id: GuildId,
    user_id: UserId,
    reason: Option<&str>,
) -> Result<UserId, Error> {
    http.as_ref().kick_member(guild_id, user_id, reason).await?;
    Ok(user_id)
}
