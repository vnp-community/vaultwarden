# Start  
docker run -d --name vaultwarden \
  -e ADMIN_TOKEN=some_random_token_as_per_above_explanation \
  -v ./data/:/data/ \
  -p 80:80 \
  vaultwarden/server:latest



docker run -d --name vaultwarden -e ADMIN_TOKEN=1fafaf1312dsfad -v ./data/:/data/  -p 80:80  vaultwarden/server:1.30.0

  