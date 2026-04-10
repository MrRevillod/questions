ifneq (,$(wildcard ./.env))
    include .env
    export
endif

SERVER=apps/server
CLIENT=apps/client
app ?= "server"
pkg ?=
service ?= $(app)
file ?=
DIAGRAMS_DIR ?= .diagrams
RENDER_DIR ?= $(DIAGRAMS_DIR)/render
KROKI ?= kroki
MMDC ?= mmdc

.DEFAULT_GOAL := run
.PHONY: run detach clean fmt lint migration machete clean-db db npmi npmu npmci setup puml mmd

setup:
	rm -rf $(CLIENT)/node_modules
	cd $(CLIENT) && corepack pnpm install --frozen-lockfile --ignore-scripts
	docker compose build
	docker compose up client -d --force-recreate --renew-anon-volumes

run:
	docker compose up

detach:
	docker compose up -d

clean:
	docker compose down -v

clean-db:
	docker exec questions-postgres psql -U user -d database -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
	cd $(SERVER) && sqlx migrate run --source ./config/migrations --database-url $(LOCAL_POSTGRES_DATABASE_URL)

db:
	docker exec -it questions-postgres /bin/bash -c "psql -U user -d database"

fmt:
	cargo fmt --all
	cd $(CLIENT) && corepack pnpm run format

lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings
	cd $(CLIENT) && corepack pnpm run lint

migration:
	cd $(SERVER) && sqlx migrate add --source ./config/migrations "$(name)"

machete:
	cargo machete

npmi:
	cd apps/$(app) && corepack pnpm add $(pkg)
	docker compose exec $(service) /bin/sh -c "cd /app/apps/$(app) && corepack pnpm add $(pkg)"

npmu:
	cd apps/$(app) && corepack pnpm remove $(pkg)
	docker compose exec $(service) /bin/sh -c "cd /app/apps/$(app) && corepack pnpm remove $(pkg)"

npmci:
	cd apps/$(app) && corepack pnpm install --frozen-lockfile --ignore-scripts
	docker compose exec $(service) /bin/sh -c "cd /app/apps/$(app) && corepack pnpm install --frozen-lockfile --ignore-scripts"

puml:
	@test -n "$(file)" || (echo 'Uso: make puml file=nombre' && exit 1)
	@test -f "$(DIAGRAMS_DIR)/$(file).puml" || (echo "No existe $(DIAGRAMS_DIR)/$(file).puml" && exit 1)
	@command -v "$(KROKI)" >/dev/null 2>&1 || (echo "No se encontro kroki. Usa KROKI=/ruta/al/binario" && exit 1)
	@mkdir -p "$(RENDER_DIR)"
	@"$(KROKI)" convert "$(DIAGRAMS_DIR)/$(file).puml" --out-file "$(RENDER_DIR)/$(file).png"
	@echo "Generado $(RENDER_DIR)/$(file).png"

mmd:
	@test -n "$(file)" || (echo 'Uso: make mmd file=nombre' && exit 1)
	@test -f "$(DIAGRAMS_DIR)/$(file).mmd" || (echo "No existe $(DIAGRAMS_DIR)/$(file).mmd" && exit 1)
	@command -v "$(MMDC)" >/dev/null 2>&1 || (echo "No se encontro mmdc. Usa MMDC=/ruta/al/binario" && exit 1)
	@mkdir -p "$(RENDER_DIR)"
	@"$(MMDC)" -i "$(DIAGRAMS_DIR)/$(file).mmd" -o "$(RENDER_DIR)/$(file).png" -b white
	@echo "Generado $(RENDER_DIR)/$(file).png"
