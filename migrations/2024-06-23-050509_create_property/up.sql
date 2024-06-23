-- Your SQL goes here

CREATE TABLE "property"(
	"property_id" UUID NOT NULL PRIMARY KEY,
	"property_name" VARCHAR NOT NULL,
	"property_password" VARCHAR NOT NULL,
	"property_email" VARCHAR NOT NULL,
	"property_phone" VARCHAR NOT NULL
);
