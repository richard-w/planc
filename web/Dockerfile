FROM node:16

ARG uid
ARG gid

# Install angular CLI
RUN npm install -g @angular/cli

# Add user and group
RUN \
  userdel -r node && \
  groupadd -g ${gid} user && \
  useradd -m -u ${uid} -g ${gid} user
USER user:user
