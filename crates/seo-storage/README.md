
## Generate Entities

```sh
echo DATABASE_URL="sqlite:./sqlite.db?mode=rwc" >> .env

```

- Generate migration
  ```sh
  sea-orm-cli migrate generate MIGRATION_NAME
  ```

- Generate entities
  ```sh
  sea-orm-cli migrate refresh
  sea-orm-cli generate entity -o src/entitie
  ```