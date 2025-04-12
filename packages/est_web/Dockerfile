# This Dockerfile assumes the build is already done
# and the output is in the .output directory
# with node_modules ready

FROM node:20-alpine AS runtime
WORKDIR /usr/src/app

COPY .output .

ENV HOST 0.0.0.0
ENV PORT 8000
EXPOSE 8000
ENTRYPOINT [ "node", "server/index.mjs" ]
