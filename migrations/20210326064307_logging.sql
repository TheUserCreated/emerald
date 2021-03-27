-- Add migration script here
CREATE TABLE public.logging
(
    guild_id bigint NOT NULL,
    channel_id bigint NOT NULL,
    channel_create bool NOT NULL,
    channel_update bool NOT NULL,
    ban_add bool NOT NULL,
    ban_remove bool NOT NULL,
    member_join bool NOT NULL,
    member_remove bool NOT NULL,
    role_create bool NOT NULL,
    role_update bool NOT NULL,
    role_delete bool NOT NULL,
    invite_create bool NOT NULL,
    invite_delete bool NOT NULL,
    message_edit bool NOT NULL,
    message_delete bool NOT NULL,
    message_delete_bulk bool NOT NULL,
    webhook_update bool NOT NULL,
    PRIMARY KEY (guild_id)


);
ALTER TABLE public.logging
    OWNER to emerald;