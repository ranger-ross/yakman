FROM node:20 as builder

WORKDIR /app

# Install dependencies
COPY ./frontend/package*.json ./
RUN npm install

# Copy project source code
COPY ./frontend ./
# Remove any files that are used for local dev that should not be included in the production build
RUN rm -rf build
RUN rm .env
RUN rm -rf .svelte-kit


# Build application
RUN npm run build


FROM node:20-alpine

WORKDIR /app

COPY --from=builder /app/package*.json  /app/
COPY --from=builder /app/build /app/build

RUN npm install --production

ENV YAKMAN_API_URL ''

CMD ["node", "build"]