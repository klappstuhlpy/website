# klappstuhl.me Website

This is my personal website for informations and image hosting.

## Prerequisites

Before running the website, make sure you have the following installed:

- Rust ~1.84.0: [Download Rust](https://www.rust-lang.org/tools/install)

## Installation

1. **Clone the repository:**

```bash
git clone https://github.com/klappstuhlpy/klappstuhl_me.git
```

2. **Create a PostgreSQL database for the website:**

- Launch the PostgreSQL command-line interface.
- Run the following command to create a new database:

```sql
CREATE ROLE percy WITH LOGIN PASSWORD 'password';
CREATE DATABASE percy OWNER percy;
CREATE EXTENSION pg_trgm;

CREATE TABLE IF NOT EXISTS images
(
    id         text  NOT NULL PRIMARY KEY,
    image_data bytea NOT NULL,
    mimetype   text  NOT NULL
);
```

3. **Build and run the project:**

```bash
cargo build
cargo run
```

## License

This project is licensed under the MPL License. See the LICENSE file for details.
