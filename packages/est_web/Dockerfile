# This Dockerfile assume the build is already done
# and the output is in the .output directory

FROM oven/bun:1-distroless AS runtime
WORKDIR /usr/src/app

COPY .output .
RUN bun install --production --cwd server

ENV HOST 0.0.0.0
ENV PORT 8000
EXPOSE 8000
ENTRYPOINT [ "bun", "run", "server/index.mjs" ]
