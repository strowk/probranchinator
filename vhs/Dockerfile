FROM ghcr.io/charmbracelet/vhs
RUN apt-get update && apt-get install -y curl sudo
ARG USERNAME=vhs-user
ARG USER_UID=1000
ARG USER_GID=$USER_UID

# Create the user
RUN groupadd --gid $USER_GID $USERNAME \
    && useradd --uid $USER_UID --gid $USER_GID -m $USERNAME \
    #
    # [Optional] Add sudo support. Omit if you don't need to install software after connecting.
    && echo $USERNAME ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/$USERNAME \
    && chmod 0440 /etc/sudoers.d/$USERNAME
RUN curl -s https://raw.githubusercontent.com/strowk/probranchinator/main/install.sh | bash
RUN apt-get install -y git
USER $USERNAME
