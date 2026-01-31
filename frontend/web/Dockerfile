# Build Stage
FROM node:20-slim as build
WORKDIR /app

# Copy package files for dependency caching
COPY frontend/web/package.json frontend/web/pnpm-lock.yaml ./
RUN npm install -g pnpm && pnpm install --frozen-lockfile

# Copy source code
COPY frontend/web/ .

# Build-time environment variables
ARG VITE_API_URL=/api
ENV VITE_API_URL=$VITE_API_URL

# Build the application
RUN pnpm run build

# Serve Stage
FROM nginx:alpine

# Copy custom nginx configuration for SPA routing and API proxy
COPY frontend/web/nginx.conf /etc/nginx/conf.d/default.conf

# Copy built assets
COPY --from=build /app/dist /usr/share/nginx/html

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost/nginx-health || exit 1

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
