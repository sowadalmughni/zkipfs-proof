# Build Stage
FROM node:20-slim as build
WORKDIR /app
COPY frontend/web/package.json ./
COPY frontend/web/package.json ./
RUN npm install -g pnpm && pnpm install
COPY frontend/web/ .
RUN pnpm run build

# Serve Stage
FROM nginx:alpine
COPY --from=build /app/dist /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
