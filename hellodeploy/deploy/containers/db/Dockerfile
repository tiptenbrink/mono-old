FROM postgres:16-bookworm
ADD postgresql.conf /conf_dir/postgresql.conf
ADD pg_hba.conf /conf_dir/pg_hba.conf
CMD ["postgres", "-c", "config_file=/conf_dir/postgresql.conf"]