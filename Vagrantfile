VAGRANTFILE_API_VERSION = "2"

Vagrant.configure(VAGRANTFILE_API_VERSION) do |config|
    config.vm.hostname = "fedora41"
    config.vm.box = "fedora/41-cloud-base"
    config.vm.box_version = "41-20241024.0"

    config.vm.provision "shell", inline: "mkdir -p /home/vagrant/ocibuilder"
    config.vm.synced_folder ".", "/home/vagrant/ocibuilder",
        type: "nfs",
        nfs_version: 4,
        nfs_udp: false

    config.vm.provider :libvirt do |domain|
        domain.memory = 4096
        domain.cpus = 2
    end

    setup_env = <<-BASH
dnf -y update
dnf -y install cargo rustc bats
BASH

    config.vm.provision "shell", inline: setup_env
    config.vm.provision "shell", inline: "chown -R vagrant:vagrant /home/vagrant/ocibuilder"

end
