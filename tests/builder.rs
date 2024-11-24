#[cfg(test)]
mod tests {
    use ocibuilder::{builder::oci::OCIBuilder, utils};
    use serial_test::serial;

    // image pull
    #[tokio::test]
    #[serial]
    async fn image_pull() {
        let root_dir_path = utils::get_root_dir(None);
        let builder = OCIBuilder::new(root_dir_path).expect("oci builder");

        let image_name = "quay.io/quay/busybox:latest";
        builder
            .pull(image_name, &false, &true)
            .await
            .expect("image to be pulled");

        let store_images = builder.image_store().images().expect("list of images");
        let mut image_found = false;
        for simage in store_images {
            let simage_name = format!("{}:{}", simage.repository(), simage.tag());
            if simage_name == image_name {
                image_found = true;

                break;
            }
        }

        if !image_found {
            assert!(true, "image not found in datastore")
        }
    }

    // image remove
    #[tokio::test]
    #[serial]
    async fn image_rm() {
        let root_dir_path = utils::get_root_dir(None);
        let builder = OCIBuilder::new(root_dir_path).expect("oci builder");

        let image_name: &str = "docker.io/library/httpd:latest";
        let container_name = "oci_builder_test_rmi";
        builder
            .from(image_name, Some(container_name.to_string()), &false, &true)
            .await
            .expect("container from an image");

        match builder.rmi(&[image_name.to_string()], &false) {
            Ok(_) => assert!(true, "was accepting error while image is in use"),
            Err(_err) => {}
        }

        builder
            .rmi(&[image_name.to_string()], &true)
            .expect("image removal");

        let store_images = builder.image_store().images().expect("list of images");
        let mut image_found = false;
        for simage in store_images {
            let simage_name = format!("{}:{}", simage.repository(), simage.tag());
            if simage_name == image_name {
                image_found = true;

                break;
            }
        }

        if image_found {
            assert!(true, "image still found in datastore")
        }
    }

    // container from scratch
    #[tokio::test]
    #[serial]
    async fn container_from_scratch() {
        let root_dir_path = utils::get_root_dir(None);
        let builder = OCIBuilder::new(root_dir_path).expect("oci builder");

        let image_name: &str = "scratch";
        let container_name = "oci_builder_test_from_scratch";
        builder
            .from(image_name, Some(container_name.to_string()), &false, &true)
            .await
            .expect("container from scratch");

        let store_containers = builder
            .container_store()
            .containers()
            .expect("list of containers");

        let mut container_found = false;
        for scontainer in store_containers {
            if scontainer.image_name() == image_name && scontainer.name() == container_name {
                container_found = true;

                break;
            }
        }

        if !container_found {
            assert!(true, "container not found in datastore")
        }
    }

    // container from
    #[tokio::test]
    #[serial]
    async fn container_from() {
        let root_dir_path = utils::get_root_dir(None);
        let builder = OCIBuilder::new(root_dir_path).expect("oci builder");

        let image_name: &str = "quay.io/quay/busybox:latest";
        let container_name = "oci_builder_test_from";
        builder
            .from(image_name, Some(container_name.to_string()), &false, &true)
            .await
            .expect("container from an image");

        let store_containers = builder
            .container_store()
            .containers()
            .expect("list of containers");

        let mut container_found = false;
        for scontainer in store_containers {
            if scontainer.image_name() == image_name && scontainer.name() == container_name {
                container_found = true;

                break;
            }
        }

        if !container_found {
            assert!(true, "container not found in datastore")
        }
    }

    // container remove
    #[tokio::test]
    #[serial]
    async fn container_rm() {
        let root_dir_path = utils::get_root_dir(None);
        let builder = OCIBuilder::new(root_dir_path).expect("oci builder");

        let image_name: &str = "quay.io/quay/busybox:latest";
        let container_name = "oci_builder_test_rm";
        builder
            .from(image_name, Some(container_name.to_string()), &false, &true)
            .await
            .expect("container from an image");

        let store_containers = builder
            .container_store()
            .containers()
            .expect("list of containers");

        let mut container_found = false;
        for scontainer in store_containers {
            if scontainer.image_name() == image_name && scontainer.name() == container_name {
                container_found = true;

                break;
            }
        }

        if !container_found {
            assert!(true, "container not found in datastore")
        }

        // now the container is created remove it
        builder
            .rm(&[container_name.to_string()])
            .expect("container to be removed");

        let store_containers = builder
            .container_store()
            .containers()
            .expect("list of containers");

        let mut container_found = false;
        for scontainer in store_containers {
            if scontainer.image_name() == image_name && scontainer.name() == container_name {
                container_found = true;

                break;
            }
        }

        if container_found {
            assert!(true, "container still found in datastore after removal")
        }
    }
}
