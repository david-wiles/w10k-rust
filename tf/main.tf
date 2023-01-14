terraform {
  required_version = ">= 1.0.0"
}

resource "digitalocean_droplet" "w10k-rust" {
  image      = "ubuntu-22-10-x64"
  name       = "w10k-rust"
  region     = "nyc1"
  size       = "s-1vcpu-1gb"
  ssh_keys   = [data.digitalocean_ssh_key.do.id]
  monitoring = true

  connection {
    host        = self.ipv4_address
    user        = "root"
    type        = "ssh"
    private_key = file(var.pvt_key)
    timeout     = "2m"
  }

  provisioner "remote-exec" {
    inline = [
      "sudo apt-get update",
      // scp executables
      "git clone https://github.com/david-wiles/w10k-rust.git"
    ]
  }
}

resource "digitalocean_domain" "default" {
  name       = format("w10k-rust.%s", var.domain)
  ip_address = digitalocean_droplet.w10k-rust.ipv4_address
}

