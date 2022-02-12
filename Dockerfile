# Debian based image with the lastest stable version of Rust.
FROM rust:slim-bullseye AS multistage

# The executable binary must be in the same folder as the Dockerfile.
COPY faithful-servant-bot /opt/

# Create a new container, which will be used to actually run the bot.
FROM rust:slim-bullseye

# Create the folder where the bot will run.
RUN cd /opt && \
        mkdir botdir

# Copy from the initial container.
COPY --from=multistage /opt/faithful-servant-bot /opt/botdir/

# Make the file executale
RUN chmod +x /opt/botdir/faithful-servant-bot

# Add the user, chown folder.
RUN useradd --no-log-init --shell /bin/false --no-create-home andrew-martin && \
        chown -R andrew-martin:andrew-martin /opt/botdir

# Change user, set workdir.
USER andrew-martin
WORKDIR /opt/botdir/

# Start the bot.
ENTRYPOINT ["./faithful-servant-bot"]
