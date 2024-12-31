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

export AWS_PAGER=""
export REPO="retpolanne/ebs-autoscale-rust"
aws sts get-caller-identity || exit 1

OIDC_PROVIDER=$(aws iam create-open-id-connect-provider \
    --url "https://token.actions.githubusercontent.com" \
    --thumbprint-list "6938fd4d98bab03faadb97b34396831e3780aea1" \
    --client-id-list 'sts.amazonaws.com' | jq -r '.OpenIDConnectProviderArn')

if [ "$OIDC_PROVIDER" != "" ]; then
    TEMP=$(mktemp)
    cat <<EOF> "$TEMP"
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Principal": {
                "Federated": "$OIDC_PROVIDER"
            },
            "Action": "sts:AssumeRoleWithWebIdentity",
            "Condition": {
                "StringEquals": {
                    "token.actions.githubusercontent.com:sub": "repo:$REPO:ref:refs/heads/main",
                    "token.actions.githubusercontent.com:aud": "sts.amazonaws.com"
                }
            }
        }
    ]
}
EOF

    ROLE_ARN=$(aws iam create-role --role-name GitHubAction-AssumeRoleWithAction --assume-role-policy-document "file://$TEMP" | jq -r '.Role.Arn')
    gh variable set AWS_GH_ROLE_ARN --body "$ROLE_ARN"
fi

aws iam attach-role-policy --role-name GithubAction-AssumeRoleWithAction --policy-arn "arn:aws:iam::aws:policy/AmazonEC2FullAccess"
aws ec2 create-security-group --description "GitHub CI" --group-name github-ci
aws ec2 authorize-security-group-ingress \
    --group-name github-ci \
    --ip-permissions IpProtocol=tcp,FromPort=2020,ToPort=2020,IpRanges="[{CidrIp=$(curl -4 ifconfig.io)/32}]"
