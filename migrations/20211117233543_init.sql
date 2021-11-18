-- Add migration script here

create schema if not exists axumstarter;

create table axumstarter.user(
    id uuid primary key,
    username text not null,
    email text not null,

    unique(username)
)
