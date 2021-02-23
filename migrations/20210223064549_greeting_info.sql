-- Add migration script here
CREATE TABLE public.greeting_info
(
    guild_id bigint NOT NULL,
    channel_id bigint NOT NULL,
    role_id bigint NOT NULL,
    greeting text NOT NULL,

    PRIMARY KEY (guild_id)

);
ALTER TABLE public.greeting_info
    OWNER to emerald;