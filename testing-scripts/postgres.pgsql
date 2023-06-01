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


