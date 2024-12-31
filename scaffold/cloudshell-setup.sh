#!/bin/bash
#
# Copyright 2024 Anne Isabelle Macedo.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
# https://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

# This script will create a user for github-ci,
# set up OIDC provider for GitHub,
# and set up needed permissions for GitHub

# This script sets up AWS cloudshell for provisioning.

export REPO="retpolanne/ebs-autoscale-rust"
sudo yum install yum-utils -y
sudo yum-config-manager --add-repo https://cli.github.com/packages/rpm/gh-cli.repo
sudo yum install gh -y

gh auth login
gh repo clone "$REPO"
