// Copyright (c) 2022 Tomohide Takao
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    identify_unused_iam::run().await
}
