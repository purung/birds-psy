dev:
	zellij run -f -- direnv exec . cargo-leptos watch --hot-reload

docker:
	zellij run -f -- direnv exec . docker compose up --build

pg:
	zellij run -f -- direnv exec . docker compose -f docker-compose-pg.yml up
