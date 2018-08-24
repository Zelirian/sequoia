use std::fmt;

use OnePassSig;
use Packet;
use KeyID;
use HashAlgorithm;
use PublicKeyAlgorithm;
use SignatureType;
use serialize::Serialize;

impl fmt::Debug for OnePassSig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("OnePassSig")
            .field("version", &self.version)
            .field("sigtype", &self.sigtype)
            .field("hash_algo", &self.hash_algo)
            .field("pk_algo", &self.pk_algo)
            .field("issuer", &self.issuer)
            .field("last", &self.last)
            .finish()
    }
}

impl PartialEq for OnePassSig {
    fn eq(&self, other: &OnePassSig) -> bool {
        // Comparing the relevant fields is error prone in case we add
        // a field at some point.  Instead, we compare the serialized
        // versions.
        return self.to_vec() == other.to_vec();
    }
}

impl OnePassSig {
    /// Returns a new `Signature` packet.
    pub fn new(sigtype: SignatureType) ->  Self {
        OnePassSig {
            common: Default::default(),
            version: 3,
            sigtype: sigtype,
            hash_algo: HashAlgorithm::Unknown(0),
            pk_algo: PublicKeyAlgorithm::Unknown(0),
            issuer: KeyID::new(0),
            last: 1,
        }
    }

    /// Gets the signature type.
    pub fn sigtype(&self) -> SignatureType {
        self.sigtype
    }

    /// Sets the signature type.
    pub fn set_sigtype(&mut self, t: SignatureType) {
        self.sigtype = t;
    }

    /// Gets the public key algorithm.
    pub fn pk_algo(&self) -> PublicKeyAlgorithm {
        self.pk_algo
    }

    /// Sets the public key algorithm.
    pub fn set_pk_algo(&mut self, algo: PublicKeyAlgorithm) {
        self.pk_algo = algo;
    }

    /// Gets the hash algorithm.
    pub fn hash_algo(&self) -> HashAlgorithm {
        self.hash_algo
    }

    /// Sets the hash algorithm.
    pub fn set_hash_algo(&mut self, algo: HashAlgorithm) {
        self.hash_algo = algo;
    }

    /// Gets the issuer.
    pub fn issuer(&self) -> &KeyID {
        &self.issuer
    }

    /// Sets the issuer.
    pub fn set_issuer(&mut self, issuer: KeyID) {
        self.issuer = issuer;
    }

    /// Gets the last flag.
    pub fn last(&self) -> bool {
        self.last > 0
    }

    /// Sets the last flag.
    pub fn set_last(&mut self, last: bool) {
        self.last = if last { 1 } else { 0 };
    }

    /// Convert the `OnePassSig` struct to a `Packet`.
    pub fn to_packet(self) -> Packet {
        Packet::OnePassSig(self)
    }
}
