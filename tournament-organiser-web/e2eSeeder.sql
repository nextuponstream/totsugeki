-- password: securePass123#
INSERT INTO users (id, name, email, password)
VALUES ('3c3ebe96-c051-4d7c-bace-a8ddf5924cf8', 'test user', 'test@user.ch',
        '$argon2id$v=19$m=19456,t=2,p=1$CE3048RoavIzqENXTvqhNw$qdSMJcKFC4RutS/y9vievvxwH31eDrS611yUPGyKWpg'),
       ('8bc83836-d424-4698-8d65-10e288c45cca', 'Diego', 'diego@gmail.com',
        '$argon2id$v=19$m=19456,t=2,p=1$CE3048RoavIzqENXTvqhNw$qdSMJcKFC4RutS/y9vievvxwH31eDrS611yUPGyKWpg'),
       ('6775b1ef-db12-4b76-9269-49321610e0e4', 'Pink', 'pink@gmail.com',
        '$argon2id$v=19$m=19456,t=2,p=1$CE3048RoavIzqENXTvqhNw$qdSMJcKFC4RutS/y9vievvxwH31eDrS611yUPGyKWpg'),
       ('58d1e261-5481-4743-9362-ee3c6e51a6c7', 'JohnMid', 'johnmid@gmail.com',
        '$argon2id$v=19$m=19456,t=2,p=1$CE3048RoavIzqENXTvqhNw$qdSMJcKFC4RutS/y9vievvxwH31eDrS611yUPGyKWpg')
;