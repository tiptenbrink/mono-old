# We're going to run our application using Node 20 on a Debian (version 11, called 'bookworm') slim (so with very little pre-installed) container 
FROM node:20-bookworm-slim

# Here we copy the entrypoint that will be run when the container starts
COPY entrypoint.sh .

# Here you want to copy all your source files
COPY package.json .
COPY package-lock.json .

# Command to install dependencies
# This won't alter the package-lock.json
RUN npm ci

COPY app.js .

EXPOSE 8871

ENTRYPOINT ["./entrypoint.sh"]