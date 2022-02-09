-- This file should undo anything in `up.sql`
create temporary table pools_backup(id, pool_name, created_at, updated_at);

insert into pools_backup select id, pool_name, created_at, updated_at from pools;

drop table pools;

-- re-create the pools table without the additional column
create table pools (
	id integer primary key not null,
	pool_name text not null,
	created_at timestamp default current_timestamp not null,
	updated_at timestamp default current_timestamp not null,
	check (pool_name <> '')
);

insert into pools select id, pool_name, created_at, updated_at from pools_backup;

drop table pools_backup;

-- create the trigger after the backup
create trigger pools after update on pools
begin
	update pools set updated_at = current_timestamp where id = NEW.id;
end;