-- Your SQL goes here

CREATE TABLE "propertyusers"(
	"user_id" serial NOT NULL PRIMARY KEY,
	"user_name" VARCHAR NOT NULL,
	"user_password" VARCHAR NOT NULL,
	"user_role" INT4 NOT NULL,
	"property_id" UUID NOT NULL,
	FOREIGN KEY ("user_role") REFERENCES "roles"("role_id"),
	FOREIGN KEY ("property_id") REFERENCES "property"("property_id")
);

