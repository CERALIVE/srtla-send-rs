use super::Action;

impl Action {
    pub(super) const fn link_scoped(&self) -> bool {
        match self {
            Self::SetImpairment(_)
            | Self::DataBlackhole { .. }
            | Self::LinkUp(_)
            | Self::DefaultRoute(_)
            | Self::Replug
            | Self::CrossTraffic { .. } => true,
            Self::ReceiverRestart | Self::SighupReorder(_) | Self::OfferedRate { .. } => false,
            Self::Periodic { action, .. } => action.link_scoped(),
        }
    }

    pub(crate) fn same_channel(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }

    pub(super) const fn reversible(&self) -> bool {
        match self {
            Self::SetImpairment(_)
            | Self::DataBlackhole { .. }
            | Self::LinkUp(_)
            | Self::DefaultRoute(_)
            | Self::CrossTraffic { .. }
            | Self::OfferedRate { .. }
            | Self::SighupReorder(_) => true,
            Self::ReceiverRestart | Self::Replug | Self::Periodic { .. } => false,
        }
    }
}
