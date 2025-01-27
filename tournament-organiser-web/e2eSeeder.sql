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

INSERT INTO tournaments (id, name, format)
VALUES ('62aefc4c-d6ec-4c2f-98f0-b639688cbe0c', 'test tournament', 'double_elimination');

INSERT INTO tournament_organisers (tournament_id, user_id)
-- tournament 1, test user
VALUES ('62aefc4c-d6ec-4c2f-98f0-b639688cbe0c', '3c3ebe96-c051-4d7c-bace-a8ddf5924cf8');

INSERT INTO players (id, tournament_id, user_id, seeding_index)
VALUES ('d245b66d-b2cb-4a91-bab4-ce326146a7ad', '62aefc4c-d6ec-4c2f-98f0-b639688cbe0c',
        '8bc83836-d424-4698-8d65-10e288c45cca', 1),
       ('dab8c793-f134-44f5-aa7e-87fa644ba9c4', '62aefc4c-d6ec-4c2f-98f0-b639688cbe0c',
        '6775b1ef-db12-4b76-9269-49321610e0e4', 2),
       ('75e3cf69-f89b-40f1-a047-00b55701e1b0', '62aefc4c-d6ec-4c2f-98f0-b639688cbe0c',
        '58d1e261-5481-4743-9362-ee3c6e51a6c7', 3);

INSERT INTO matches (id, high_seed, high_seed_player, low_seed, low_seed_player, format, format_n)
VALUES ('ba438261-ec85-406d-9d01-585506a393fd',
        1,
        'd245b66d-b2cb-4a91-bab4-ce326146a7ad',
        2,
        null,
        'first_to_n',
        3),
       ('dda1e3cc-dff9-4976-9053-912e1389489f',
        2,
        'dab8c793-f134-44f5-aa7e-87fa644ba9c4',
        3,
        '75e3cf69-f89b-40f1-a047-00b55701e1b0',
        'first_to_n',
        3),
       ('4b73bc23-b06e-42bf-a271-350b1031c61a',
        2,
        null,
        3,
        null,
        'first_to_n',
        3),
       ('8b22810c-4ee8-4624-82a1-f30ec8f173e0',
        1,
        null,
        2,
        null,
        'first_to_n',
        3),
       ('e280e49d-21c6-4769-b7f7-c6c664dcf199',
        1,
        null,
        2,
        null,
        'first_to_n',
        3);

INSERT INTO tournament_matches (tournament_id, match_id, pos)
VALUES ('62aefc4c-d6ec-4c2f-98f0-b639688cbe0c',
        'ba438261-ec85-406d-9d01-585506a393fd',
        1),
       ('62aefc4c-d6ec-4c2f-98f0-b639688cbe0c',
        'dda1e3cc-dff9-4976-9053-912e1389489f',
        2),
       ('62aefc4c-d6ec-4c2f-98f0-b639688cbe0c',
        '4b73bc23-b06e-42bf-a271-350b1031c61a',
        3),
       ('62aefc4c-d6ec-4c2f-98f0-b639688cbe0c',
        '8b22810c-4ee8-4624-82a1-f30ec8f173e0',
        4),
       ('62aefc4c-d6ec-4c2f-98f0-b639688cbe0c',
        'e280e49d-21c6-4769-b7f7-c6c664dcf199',
        5);
