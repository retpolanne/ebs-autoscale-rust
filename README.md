# EBS Autoscaler Rust

This is a project inspired by the now deprecated [Amazon EBS Autoscale](https://github.com/awslabs/amazon-ebs-autoscale/tree/master). 

The purpose of this project is to reimplement the AWS EBS Autoscaler in a more robust language other than
bash, including better tests (unit testing, integration and self-tests). 


## E2E testing

This is a draft of how E2E testing should work and what needs to be covered. 

1. Build this program and serve binary - automatically generate nightly releases for testing

    Using [cargo-deb](https://crates.io/crates/cargo-deb).

2. Ensure AWS credentials are set up on the testing environment and that permissions are set up for setup, teardown

3. Ensure EC2 creation and deletion permissions are defined for testing machinery

4. Create a systemd unit for running the autoscaler as a daemon

5. Create a cloud-init script for installing the systemd unit and the program 

   5.1. on this script, add a script to fill up the block storage and ensure that the autoscaler runs

6. Teardown
