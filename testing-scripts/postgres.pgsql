create table YAK_MAN_CONFIG(
    NAME TEXT PRIMARY KEY,
    DESCRIPTION TEXT
);

create table YAK_MAN_LABEL(
    NAME TEXT PRIMARY KEY,
    DESCRIPTION TEXT
);

create table YAK_MAN_LABEL_OPTION(
    NAME TEXT NOT NULL,
    OPTION TEXT NOT NULL,
    PRIMARY KEY(NAME, OPTION),
    CONSTRAINT FK_LABEL_NAME
      FOREIGN KEY(NAME) 
	  REFERENCES YAK_MAN_LABEL(NAME)
);

create table YAK_MAN_INSTANCE_LABEL(
    INSTANCE_ID SERIAL NOT NULL,
    LABEL_NAME TEXT NOT NULL,
    OPTION TEXT NOT NULL,
    PRIMARY KEY(INSTANCE_ID, LABEL_NAME)
);

create table YAK_MAN_INSTANCE(
    INSTANCE_ID SERIAL PRIMARY KEY,
    CONFIG_NAME TEXT NOT NULL,
    DATA TEXT NOT NULL,
    CONSTRAINT FK_CONFIG_NAME
      FOREIGN KEY (CONFIG_NAME) 
	  REFERENCES YAK_MAN_CONFIG(NAME)
);


-- insert some dummy testing data

insert into yak_man_config (name, description) values ('config1', 'config description 1');
insert into yak_man_config (name, description) values ('config2', 'config description 2');
insert into yak_man_config (name, description) values ('config3', 'config description 3');

insert into yak_man_label (name, description) values ('label1', 'label1 desc');
insert into yak_man_label_option (name, option) values ('label1', 'op100');
insert into yak_man_label_option (name, option) values ('label1', 'op200');
insert into yak_man_label_option (name, option) values ('label1', 'op300');

insert into yak_man_label (name, description) values ('label2', 'label2 desc');
insert into yak_man_label_option (name, option) values ('label2', 'op1');
insert into yak_man_label_option (name, option) values ('label2', 'op2');
insert into yak_man_label_option (name, option) values ('label2', 'op3');

insert into yak_man_INSTANCE (instance_id, config_name, data) values (1, 'config1', 'this is my data');
insert into yak_man_INSTANCE (instance_id, config_name, data) values (2, 'config1', 'this is my data with labels');

insert into YAK_MAN_INSTANCE_LABEL (instance_id, label_name, option) values (2, 'label1', 'op100');
insert into YAK_MAN_INSTANCE_LABEL (instance_id, label_name, option) values (2, 'label2', 'op2');


-- Testing queries

SELECT name, option FROM yak_man_LABEL_OPTION where name = 'label1';

select * from yak_man_INSTANCE;

SELECT 
  i.config_name, 
  i.instance_id, 
  STRING_AGG(l.option, ', ') AS options
FROM 
  yak_man_instance i 
  LEFT JOIN yak_man_instance_label l 
    ON i.instance_id = l.instance_id
WHERE
  config_name = 'config1'
GROUP BY 
  i.config_name, 
  i.instance_id;

