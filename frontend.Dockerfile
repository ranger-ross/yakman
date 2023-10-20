FROM node:21 as builder
# Add PNPM
RUN corepack enable

WORKDIR /app

# Install dependencies
COPY ./frontend/package*.json ./
RUN pnpm install

# Copy project source code
COPY ./frontend ./
# Remove any files that are used for local dev that should not be included in the production build
RUN rm -rf build
RUN rm .env
RUN rm -rf .svelte-kit


# Build application
RUN pnpm run build


FROM node:21-alpine
# Add PNPM
RUN corepack enable

WORKDIR /app

COPY --from=builder /app/package*.json  /app/
COPY --from=builder /app/build /app/build

RUN pnpm install --production

ENV YAKMAN_API_URL ''

CMD ["node", "build"]