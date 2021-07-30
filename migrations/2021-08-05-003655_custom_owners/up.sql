-- Your SQL goes here
create table custom_owners (
	id integer primary key not null,
	custom_owner_name text not null,
	recipient text not null,
	description text,
	created_at timestamp default current_timestamp not null,
	updated_at timestamp default current_timestamp not null,
	check (custom_owner_name <> '')
	check (recipient <> '')
);

create trigger custom_owners after update on custom_owners
begin
	update custom_owners set updated_at = current_timestamp where id = NEW.id;
end;