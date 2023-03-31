create table CONFIG_MAN_CONFIG(
    NAME TEXT PRIMARY KEY,
    DESCRIPTION TEXT
);

create table CONFIG_MAN_LABEL(
    NAME TEXT PRIMARY KEY,
    DESCRIPTION TEXT
);

create table CONFIG_MAN_LABEL_OPTION(
    NAME TEXT NOT NULL,
    OPTION TEXT NOT NULL,
    PRIMARY KEY(NAME, OPTION),
    CONSTRAINT FK_LABEL_NAME
      FOREIGN KEY(NAME) 
	  REFERENCES CONFIG_MAN_LABEL(NAME)
);

create table CONFIG_MAN_INSTANCE_LABEL(
    INSTANCE_ID SERIAL NOT NULL,
    LABEL_NAME TEXT NOT NULL,
    OPTION TEXT NOT NULL,
    PRIMARY KEY(INSTANCE_ID, LABEL_NAME)
);

create table CONFIG_MAN_INSTANCE(
    INSTANCE_ID SERIAL PRIMARY KEY,
    CONFIG_NAME TEXT NOT NULL,
    DATA TEXT NOT NULL,
    CONSTRAINT FK_CONFIG_NAME
      FOREIGN KEY (CONFIG_NAME) 
	  REFERENCES CONFIG_MAN_CONFIG(NAME)
);


-- insert some dummy testing data

insert into config_man_config (name, description) values ('config1', 'config description 1');
insert into config_man_config (name, description) values ('config2', 'config description 2');
insert into config_man_config (name, description) values ('config3', 'config description 3');

insert into config_man_label (name, description) values ('label1', 'label1 desc');
insert into config_man_label_option (name, option) values ('label1', 'op100');
insert into config_man_label_option (name, option) values ('label1', 'op200');
insert into config_man_label_option (name, option) values ('label1', 'op300');

insert into config_man_label (name, description) values ('label2', 'label2 desc');
insert into config_man_label_option (name, option) values ('label2', 'op1');
insert into config_man_label_option (name, option) values ('label2', 'op2');
insert into config_man_label_option (name, option) values ('label2', 'op3');


SELECT name, option FROM CONFIG_MAN_LABEL_OPTION where name = 'label1';
