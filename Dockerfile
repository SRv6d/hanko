FROM scratch

LABEL org.opencontainers.image.title="hanko"
LABEL org.opencontainers.image.authors="Marvin Vogt <m@rvinvogt.com>"

COPY hanko /app/hanko

ENTRYPOINT ["/app/hanko"]
