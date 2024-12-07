use ratatui::layout::Constraint;

#[derive(Debug)]
pub enum DisplayType {
    Gb,
    Digits,
}

#[derive(Debug)]
pub struct Field {
    pub name: &'static str,
    pub dataref: &'static str,
    pub field_type: DisplayType,
    pub constraint: Constraint,
}

/// for 3-digit fields (5 chars gives some padding)
const _WIDTH_3: u16 = 10;
/// for 4-digit fields
const _WIDTH_4: u16 = 10;
/// for 5-digit fields
const _WIDTH_5: u16 = 10;
/// for G/B fields
const _WIDTH_GB: u16 = 10;

/// MCP A fields
pub const MCP_A_FIELDS: [Field; 11] = [
    Field {
        name: "CRS",
        dataref: "laminar/B738/autopilot/course_pilot",
        field_type: DisplayType::Digits,
        constraint: Constraint::Length(_WIDTH_3),
    },
    Field {
        name: "A/T",
        dataref: "laminar/B738/autopilot/autothrottle_arm_pos",
        field_type: DisplayType::Gb,
        constraint: Constraint::Length(_WIDTH_GB),
    },
    Field {
        name: "IAS",
        dataref: "laminar/B738/autopilot/airspeed",
        field_type: DisplayType::Digits,
        constraint: Constraint::Length(_WIDTH_4),
    },
    Field {
        name: "VNAV",
        dataref: "laminar/B738/autopilot/vnav_status1",
        field_type: DisplayType::Gb,
        constraint: Constraint::Length(_WIDTH_GB),
    },
    Field {
        name: "HDG",
        dataref: "laminar/B738/autopilot/mcp_hdg_dial",
        field_type: DisplayType::Digits,
        constraint: Constraint::Length(_WIDTH_3),
    },
    Field {
        name: "LNAV",
        dataref: "laminar/B738/autopilot/lnav_status",
        field_type: DisplayType::Gb,
        constraint: Constraint::Length(_WIDTH_GB),
    },
    Field {
        name: "ALT",
        dataref: "laminar/B738/autopilot/mcp_alt_dial",
        field_type: DisplayType::Digits,
        constraint: Constraint::Length(_WIDTH_5),
    },
    Field {
        name: "V/S",
        dataref: "sim/cockpit/autopilot/vertical_velocity",
        field_type: DisplayType::Digits,
        constraint: Constraint::Length(_WIDTH_5),
    },
    Field {
        name: "CMD A",
        dataref: "laminar/B738/autopilot/cmd_a_status",
        field_type: DisplayType::Gb,
        constraint: Constraint::Length(_WIDTH_GB),
    },
    Field {
        name: "CMD B",
        dataref: "laminar/B738/autopilot/cmd_b_status",
        field_type: DisplayType::Gb,
        constraint: Constraint::Length(_WIDTH_GB),
    },
    Field {
        name: "CRS",
        dataref: "laminar/B738/autopilot/course_copilot",
        field_type: DisplayType::Digits,
        constraint: Constraint::Length(_WIDTH_3),
    },
];

/// MCP B fields
pub const MCP_B_FIELDS: [Field; 11] = [
    Field {
        name: "F/D",
        dataref: "laminar/B738/autopilot/flight_director_pos",
        field_type: DisplayType::Gb,
        constraint: Constraint::Length(_WIDTH_GB),
    },
    Field {
        name: "N1",
        dataref: "laminar/B738/autopilot/n1_status",
        field_type: DisplayType::Gb,
        constraint: Constraint::Length(_WIDTH_GB),
    },
    Field {
        name: "SPEED",
        dataref: "laminar/B738/autopilot/speed_status1",
        field_type: DisplayType::Gb,
        constraint: Constraint::Length(_WIDTH_GB),
    },
    Field {
        name: "LVL CHG",
        dataref: "laminar/B738/autopilot/lvl_chg_status",
        field_type: DisplayType::Gb,
        constraint: Constraint::Length(_WIDTH_GB),
    },
    Field {
        name: "HDG SEL",
        dataref: "laminar/B738/autopilot/hdg_sel_status",
        field_type: DisplayType::Gb,
        constraint: Constraint::Length(_WIDTH_GB),
    },
    Field {
        name: "APP",
        dataref: "laminar/B738/autopilot/app_status",
        field_type: DisplayType::Gb,
        constraint: Constraint::Length(_WIDTH_GB),
    },
    Field {
        name: "ALT HLD",
        dataref: "laminar/B738/autopilot/alt_hld_status",
        field_type: DisplayType::Gb,
        constraint: Constraint::Length(_WIDTH_GB),
    },
    Field {
        name: "V/S",
        dataref: "laminar/B738/autopilot/vs_status",
        field_type: DisplayType::Gb,
        constraint: Constraint::Length(_WIDTH_GB),
    },
    Field {
        name: "CWS A",
        dataref: "laminar/B738/autopilot/cws_a_status",
        field_type: DisplayType::Gb,
        constraint: Constraint::Length(_WIDTH_GB),
    },
    Field {
        name: "CWS B",
        dataref: "laminar/B738/autopilot/cws_b_status",
        field_type: DisplayType::Gb,
        constraint: Constraint::Length(_WIDTH_GB),
    },
    Field {
        name: "F/D",
        dataref: "laminar/B738/autopilot/flight_director_fo_pos",
        field_type: DisplayType::Gb,
        constraint: Constraint::Length(_WIDTH_GB),
    },
];

fn main() { }