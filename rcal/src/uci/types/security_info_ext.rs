//! Extension methods for [`SecurityInformationType`]
//!
//! The shape of `SecurityInformationType` differs between schemas (e.g. UCI 2.5 vs. the
//! UCI 2.3-based A-GRA schema). `build.rs` inspects the generated fields and sets
//! `uci_security_cui_basic` / `uci_sci_controls_choice` so this code matches the schema.

#[cfg(uci_sci_controls_choice)]
use super::SCI_ControlsChoiceType_;
use super::{ReleasableToChoiceType_, SecurityInformationType_};

/// Extension trait for [`SecurityInformation`]
pub trait SecurityInformationExt {
    /// Render a US classification banner string from a [`SecurityInformationType`].
    ///
    /// Format: `CLASSIFICATION[//SCI1/SCI2][//SAR-ID][//DISSEM1/DISSEM2][ TO REL1, REL2]`
    ///
    /// If the schema has `CUI_Basic`, it is non-empty, and `Classification` is `U`, the
    /// banner begins with `CUI` instead.  Dissemination controls and releasable-to entries
    /// that carry no string value (`EnumNotSet`) are silently omitted.
    fn security_banner(&self) -> String;
}

impl SecurityInformationExt for SecurityInformationType_ {
    fn security_banner(&self) -> String {
        use ReleasableToChoiceType_;

        let class_str = self.classification.as_str().unwrap_or("U");
        #[cfg(uci_security_cui_basic)]
        let cui = !self.cuibasic.is_empty();
        #[cfg(not(uci_security_cui_basic))]
        let cui = false;
        let banner_class = if cui && class_str == "U" {
            "CUI"
        } else {
            class_str
        };

        let mut banner = banner_class.to_string();

        // SCI controls: //SCI1/SCI2/…
        #[cfg(not(uci_sci_controls_choice))]
        let sci: Vec<&str> = self.scicontrols.iter().filter_map(|v| v.as_str()).collect();
        #[cfg(uci_sci_controls_choice)]
        let sci: Vec<&str> = self
            .scicontrols
            .iter()
            .filter_map(|v| match v {
                SCI_ControlsChoiceType_::StandardCompartment { inner } => inner.as_str(),
                SCI_ControlsChoiceType_::SubCompartment { inner } => {
                    (!inner.is_empty()).then_some(inner.as_str())
                }
            })
            .collect();
        if !sci.is_empty() {
            banner.push_str("//");
            banner.push_str(&sci.join("/"));
        }

        // SAR identifiers: //SAR-Id (each its own section)
        for sar in &self.saridentifier {
            if !sar.is_empty() {
                banner.push_str("//");
                banner.push_str(sar.as_str());
            }
        }

        // Dissemination controls: //DISSEM1/DISSEM2/…
        let dissem: Vec<&str> = self
            .dissemination_controls
            .iter()
            .filter_map(|v| v.as_str())
            .collect();
        if !dissem.is_empty() {
            banner.push_str("//");
            banner.push_str(&dissem.join("/"));
        }

        // Releasable-to: <space>TO REL1, REL2
        let rel: Vec<String> = self
            .releasable_to
            .iter()
            .filter_map(|v| match v {
                ReleasableToChoiceType_::GovernmentIdentifier { inner } => {
                    inner.as_str().map(|s| s.to_string())
                }
                ReleasableToChoiceType_::NATO_SpecialWord { inner } => {
                    if inner.is_empty() {
                        None
                    } else {
                        Some(inner.clone())
                    }
                }
            })
            .collect();
        if !rel.is_empty() {
            banner.push_str(" TO ");
            banner.push_str(&rel.join(", "));
        }

        banner
    }
}

// NOTE: Every security marking and banner string in this test module is a made-up
// value for exercising `security_banner` formatting only. None of them reflect real
// classification levels, controls, or markings of any actual data.
#[cfg(test)]
mod test {
    use super::*;
    use crate::uci::types::{ClassificationEnum, DisseminationControlsEnum, ReleasableToEnum};

    #[test]
    fn plain_classification() {
        let mut si = SecurityInformationType_::default();
        si.classification = ClassificationEnum::S;
        assert_eq!(si.security_banner(), "S");
    }

    #[test]
    fn dissem_and_releasable_to() {
        let mut si = SecurityInformationType_::default();
        si.classification = ClassificationEnum::S;
        si.dissemination_controls
            .push(DisseminationControlsEnum::Nf);
        si.releasable_to
            .push(ReleasableToChoiceType_::GovernmentIdentifier {
                inner: ReleasableToEnum::Usa,
            });
        assert_eq!(si.security_banner(), "S//NF TO USA");
    }

    #[cfg(uci_security_cui_basic)]
    #[test]
    fn cui_basic_replaces_unclassified() {
        let mut si = SecurityInformationType_::default();
        si.classification = ClassificationEnum::U;
        si.cuibasic.push(crate::uci::types::CUI_BasicEnum::Adpo);
        assert_eq!(si.security_banner(), "CUI");
    }

    #[cfg(not(uci_sci_controls_choice))]
    #[test]
    fn sci_controls_enum() {
        let mut si = SecurityInformationType_::default();
        si.classification = ClassificationEnum::Ts;
        si.scicontrols
            .push(crate::uci::types::SCI_ControlsEnum::Bur);
        assert_eq!(si.security_banner(), "TS//BUR");
    }

    #[cfg(uci_sci_controls_choice)]
    #[test]
    fn sci_controls_choice_sub_compartment() {
        let mut si = SecurityInformationType_::default();
        si.classification = ClassificationEnum::Ts;
        si.scicontrols
            .push(SCI_ControlsChoiceType_::SubCompartment {
                inner: "KDK-BLFH-A1".to_string(),
            });
        assert_eq!(si.security_banner(), "TS//KDK-BLFH-A1");
    }
}
