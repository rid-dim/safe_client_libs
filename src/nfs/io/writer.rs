// Copyright 2015 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement, version 1.0.  This, along with the
// Licenses can be found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.
#[allow(dead_code)]
use nfs;
use std::sync;
use super::network_storage::NetworkStorage;
use self_encryption;
use client;

pub struct Writer {
    file: nfs::file::File,
    directory: nfs::directory_listing::DirectoryListing,
    self_encryptor: self_encryption::SelfEncryptor<NetworkStorage>,
    client: ::std::sync::Arc<::std::sync::Mutex<client::Client>>
}

impl Writer {

    pub fn new(directory: nfs::directory_listing::DirectoryListing, file: nfs::file::File,
        client: ::std::sync::Arc<::std::sync::Mutex<client::Client>>) -> Writer {
        let storage = sync::Arc::new(NetworkStorage::new(client.clone()));
        Writer {
            file: file.clone(),
            directory: directory,
            self_encryptor: self_encryption::SelfEncryptor::new(storage.clone(), file.get_datamap().clone()),
            client: client
        }
    }

    pub fn write(&mut self, data: &[u8], position: u64) {        
        self.self_encryptor.write(data, position);
    }

    pub fn close(mut self) -> Result<(), String> {
        let ref mut file = self.file;
        let ref mut directory = self.directory;
        file.set_datamap(self.self_encryptor.close());

        file.get_mut_metadata().set_modified_time(::time::now_utc());

        match directory.get_files().iter().find(|entry| entry.get_name() == file.get_name()) {
            Some(_) => {
                directory.upsert_file(file.clone());
            },
            None => {
                directory.add_file(file.clone());
            }
        };
        let mut directory_helper = nfs::helper::DirectoryHelper::new(self.client.clone());
        match directory_helper.update(directory) {
            Ok(_) => Ok(()),
            Err(_) => Err("Failed to save".to_string())
        }
    }

}
