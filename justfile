dev:
	zellij run -f -- direnv exec . cargo-leptos watch --hot-reload

docker:
	zellij run -f -- direnv exec . docker compose up 
