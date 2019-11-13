CREATE TABLE account
(
    id SERIAL NOT NULL PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    last_logged_in TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE TABLE thread
(
    id SERIAL PRIMARY KEY,
    creator SERIAL NOT NULL REFERENCES account (id),
    title TEXT NOT NULL
);

CREATE TABLE message
(
    id SERIAL,
    thread_id SERIAL REFERENCES thread(id),
    creator SERIAL NOT NULL REFERENCES account (id),
    content TEXT NOT NULL,

    PRIMARY KEY (id, thread_id)
);
