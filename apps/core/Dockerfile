FROM node:22-alpine

WORKDIR /app

COPY package.json package-lock.json ./
COPY apps/core/package.json apps/core/package.json
RUN rm package-lock.json && \
  npm install --workspace core --include-workspace-root=false --package-lock=false

COPY apps/core apps/core
RUN npm run build --workspace core

EXPOSE 3000

CMD ["npm", "run", "start", "--workspace", "core"]
