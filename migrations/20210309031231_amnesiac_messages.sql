-- Add migration script here
CREATE TABLE public.amnesiac_messages
(
    channel_id bigint NOT NULL,
    duration bigint,
    PRIMARY KEY (channel_id)


);
ALTER TABLE public.amnesiac_messages
    OWNER to emerald;