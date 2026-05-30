# Multi-stage build for the SvelteKit (adapter-node) frontend.
# Build context: the repository root.
FROM node:22-slim AS build
WORKDIR /app
RUN corepack enable
COPY package.json pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile
COPY . .
RUN pnpm run build

FROM node:22-slim
WORKDIR /app
ENV NODE_ENV=production
# adapter-node emits a self-contained server in ./build.
COPY --from=build /app/build ./build
COPY --from=build /app/package.json ./package.json
USER node
ENV PORT=3000
EXPOSE 3000
CMD ["node", "build/index.js"]
