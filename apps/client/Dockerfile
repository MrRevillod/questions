FROM node:24-alpine AS base
WORKDIR /app

RUN apk add --no-cache curl

COPY ./apps/client/package.json ./
COPY ./apps/client/pnpm-lock.yaml ./
COPY ./apps/client/.npmrc ./

RUN corepack enable && pnpm install --frozen-lockfile --ignore-scripts

FROM base AS builder
COPY ./apps/client/ ./
RUN pnpm build

FROM nginx:alpine AS prod
COPY --from=builder /app/build /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]

FROM base AS dev
ARG UID=1000
ARG GID=1000

RUN chown -R ${UID}:${GID} /app
USER ${UID}:${GID}

EXPOSE 5173

CMD ["pnpm", "dev", "--host", "0.0.0.0", "--port", "5173"]
