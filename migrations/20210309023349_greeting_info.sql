-- Add migration script here
CREATE TABLE public.greeting_info
(
    guild_id bigint NOT NULL,
    channel_id bigint NOT NULL,
    role_id bigint NOT NULL,
    greeting text NOT NULL,
    timeout bool NOT NULL,
    -- time out duration?
    PRIMARY KEY (guild_id)

);
ALTER TABLE public.greeting_info
    OWNER to emerald;