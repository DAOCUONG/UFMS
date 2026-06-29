# UFMS — UFleet Management System

Rust models for the [VDA 5050 v3.0.0](https://www.vda.de/vda-5050) fleet-control /
mobile-robot communication protocol.

The crate compiles all 9 VDA 5050 protobuf files into native Rust types via
[`prost-build`](https://crates.io/crates/prost-build), then layers an
ergonomic API on top:

- type-safe constructors for every message,
- chainable builders,
- per-topic validators that encode the spec's `if/then` rules,
- uniform header / timestamp helpers,
- geometric primitives for `Vertex2D` / polygons / NURBS trajectories,
- and an `EnumExt` trait that gives every enum `Display`, `FromStr`,
  `short_name`, and semantic helpers.

No new dependencies beyond `prost`, `prost-types`, and `prost-build`.

---

## Quick start

```rust
use ufms::prelude::*;

let header = Header::new("ACME Robotics", "amr-001")
    .with_header_id(1)
    .with_timestamp_now();

let conn = Connection::online(header.clone());
let bytes = conn.encode_to_vec();

let conn_back = Connection::decode(bytes.as_slice())?;
assert_eq!(
    ConnectionState::try_from(conn_back.connection_state)?,
    ConnectionState::ConnectionOnline,
);
```

```rust
// A speed-limit zone — the constructor enforces `maximum_speed` is set.
let zone = Zone::speed_limit(
    "z-near-charging",
    0.5,
    vec![
        Vertex2D::new(0.0, 0.0),
        Vertex2D::new(2.0, 0.0),
        Vertex2D::new(2.0, 2.0),
    ],
);
zone.validate()?;
assert!(zone.contains(Vertex2D::new(1.0, 1.0)));
```

```rust
// InstantActions builder rejects any action whose blocking_type isn't NONE.
let msg = InstantActionsBuilder::new(header)
    .add_action(Action::for_instant("pauseOrder", "ia-1"))
    .try_build()?;
```

---

## Build, test, run

```sh
cargo build        # compile the lib + demo binary
cargo test         # run the 37 integration tests
cargo run          # exercise every ergonomic surface
cargo build --release
```

`cargo build` invokes `build.rs`, which runs `prost-build` against every
proto file under `VDA5050/proto/`. Generated code lives in `OUT_DIR` and is
re-exported via `src/lib.rs`.

---

## Architecture

```
UFMS/
├── build.rs                      # prost-build invocation
├── Cargo.toml                    # prost, prost-types, prost-build only
├── VDA5050/                      # vendored v3.0.0 spec
│   ├── proto/*.proto             # the 9 input proto files
│   ├── json_schemas/*.schema     # matching JSON schemas
│   ├── assets/                   # UML / sequence diagrams
│   ├── LICENSE.txt               # upstream VDA 5050 license
│   └── README.md                 # upstream spec description
└── src/
    ├── lib.rs                    # module tree + prelude
    ├── main.rs                   # demo (cargo run)
    ├── error.rs                  # ValidationError / HeaderError / …
    ├── enums.rs                  # EnumExt trait on all 16 enums
    ├── header.rs                 # Header, HeaderExt, TimestampExt
    ├── geometry.rs               # Vertex2D, polygon_*, Trajectory
    ├── action.rs                 # Action, ActionParameter, ActionBuilder
    ├── connection.rs             # Connection lifecycle
    ├── instant_actions.rs        # InstantActionsBuilder
    ├── order.rs                  # Order/Node/Edge/Corridor
    ├── state.rs                  # State + all sub-messages
    ├── visualization.rs          # Visualization
    ├── factsheet.rs              # Factsheet + 7 sub-sections
    ├── zone_set.rs               # ZoneSet + per-zone-type Zone constructors
    └── responses.rs              # Response / Responses
```

The generated types live under `ufms::vda5050::v3::*` (re-exported via
`include!(concat!(env!("OUT_DIR"), "/vda5050.v3.rs"))`). Ergonomic helpers
are added as **inherent impls** on those generated types — no newtypes, no
parallel hierarchy. You can `use` the generated types and call ergonomic
methods on them interchangeably.

---

## Module map

| Module | Proto topic | Adds |
|---|---|---|
| [`error`](#error-handling) | (cross-cutting) | `ValidationError`, `HeaderError`, `TimestampError`, `ActionError`, `UnknownVariant` |
| [`enums`](#enums) | (cross-cutting) | `EnumExt` trait + `Display` + `FromStr` + `short_name` on all 16 enums; semantic helpers |
| [`header`](#header--timestamp) | every topic | `Header::new`, `HeaderExt` trait, `TimestampExt` |
| [`geometry`](#geometry) | common.proto | `Vertex2D` arithmetic, polygon helpers, `Trajectory::validate`, `BoundingBox` |
| [`action`](#action) | common.proto | `Action::for_instant` / `for_order` / `for_zone`; `ActionParameter` typed builders |
| [`connection`](#connection) | connection.proto | `Connection::online` / `offline` / `hibernating` / `broken` |
| [`instant_actions`](#instantactions) | instant_actions.proto | `InstantActionsBuilder` (validates `BlockingNone`) |
| [`order`](#order) | order.proto | `OrderBuilder`, `NodeBuilder`, `EdgeBuilder`, `Corridor::new` |
| [`state`](#state) | state.proto | `State::new` + 25 setters; semantic accessors; sub-message constructors |
| [`visualization`](#visualization) | visualization.proto | `Visualization::new` / `empty_for_state` |
| [`factsheet`](#factsheet) | factsheet.proto | All sub-section constructors, paired setters, validators |
| [`zone_set`](#zoneset) | zone_set.proto | Per-zone-type constructors; `Zone::validate` |
| [`responses`](#responses) | responses.proto | `Response::granted` / `queued` / `revoked` / `rejected` |

---

## Prelude

For convenience, `ufms::prelude::*` re-exports:

- Every generated type under `vda5050::v3::*`
- The ergonomic traits: `EnumExt`, `HeaderExt`, `TimestampExt`, `OrderExt`,
  `CorridorExt`, `ConnectionExt`, `ResponsesExt`, `StateExt`, `ZoneSetExt`
- The fluent builders: `ActionBuilder`, `EdgeBuilder`, `InstantActionsBuilder`,
  `LoadBuilder`, `NodeBuilder`, `OrderBuilder`, `ResponseBuilder`,
  `VisualizationBuilder`, `ZoneBuilder`, `ZoneSetBuilder`
- Error types: `ValidationError`, `HeaderError`, `TimestampError`,
  `ActionValidationError`, `UnknownVariant`
- Geometry free functions: `polygon_area`, `polygon_area_unsigned`,
  `polygon_perimeter`, `polygon_centroid`, `polygon_bounding_box`,
  `point_in_polygon`

---

## Enums

All 21 proto enums (defined in `common.proto`) get the following surface via
the `EnumExt` trait:

| Method | Behavior |
|---|---|
| `short_name()` | Lowercase, no enum-prefix form. `"online"` instead of `"CONNECTION_ONLINE"`. |
| `is_unspecified()` | True for the `0` variant. |
| `Display` impl | Prints `short_name`. |
| `FromStr` impl | Accepts `"BLOCKING_HARD"`, `"blocking_hard"`, and `"hard"`. Returns `UnknownVariant` on failure. |
| `as_str_name()` *(from prost)* | Original `"SCREAMING_SNAKE"` proto name. |

```rust
use ufms::prelude::*;

assert_eq!(BlockingType::BlockingHard.short_name(), "hard");
assert_eq!(BlockingType::BlockingHard.to_string(),     "hard");
let parsed: BlockingType = "BLOCKING_HARD".parse().unwrap();
assert_eq!(parsed, BlockingType::BlockingHard);
let parsed_lower: BlockingType = "hard".parse().unwrap();
assert_eq!(parsed_lower, BlockingType::BlockingHard);
```

### Semantic helpers

```rust
// ErrorLevel
ErrorLevel::ErrorWarning.accepts_new_orders();   // true
ErrorLevel::ErrorFatal.accepts_new_orders();     // false
ErrorLevel::ErrorCritical.can_continue_order();  // false

// EStopType
EStopType::EstopNone.is_clear();                // true

// MapStatus
MapStatus::MapEnabled.is_active();              // true

// GrantType
GrantType::GrantGranted.is_positive();          // true
GrantType::GrantRejected.is_positive();         // false

// ConnectionState
ConnectionState::ConnectionBroken.is_disconnected(); // true
```

### Enums covered

`ConnectionState`, `BlockingType`, `ActionStatus`, `OperatingMode`,
`ErrorLevel`, `InfoLevel`, `EStopType`, `MapStatus`, `OrientationType`,
`CorridorReferencePoint`, `ReleaseLossBehavior`, `ZoneType`,
`DirectedLimitation`, `BidirectedLimitation`, `RequestType`, `RequestStatus`,
`GrantType`, `ActionScope`, `ValueDataType`, `OptionalParameterSupport`,
`ActionType`.

---

## Header / Timestamp

### `Header` constructors

Every top-level message carries a `Header`. The ergonomic layer adds:

```rust
let h = Header::new("ACME", "amr-001")      // manufacturer, serial
    .with_header_id(42)                      // per-topic counter
    .with_timestamp_now()                    // spec-mandated ISO 8601 UTC ms
    .validate()?;                            // checks non-empty fields + v3.x version
```

### `HeaderExt` trait

Implemented on `Connection`, `InstantActions`, `Order`, `State`,
`Visualization`, `Factsheet`, `ZoneSet`, `Responses`. Uniform access:

```rust
let mut state = State::new(Header::new("X", "Y"));
state.set_header(Header::new("X", "Y").with_timestamp_now());
assert!(state.header().is_some());
let h = state.header_mut();           // creates empty header if missing
```

### `TimestampExt`

`prost_types::Timestamp` is an external type (orphan-rule), so we extend it
with a trait. Bring `TimestampExt` into scope to use it:

```rust
use ufms::header::TimestampExt;

let ts = prost_types::Timestamp::now_utc();
let iso = ts.to_iso8601_utc();                    // "2026-06-29T12:34:56.789Z"
let back = prost_types::Timestamp::from_iso8601_utc(&iso).unwrap();
let sys = ts.to_system_time();
let ts2 = prost_types::Timestamp::from_system_time(sys);
```

All conversions round-trip at millisecond precision, matching the
VDA 5050 wire format `YYYY-MM-DDTHH:mm:ss.fffZ`.

---

## Geometry

### `Vertex2D`

```rust
use ufms::prelude::*;
use std::ops::{Add, Sub};

let a = Vertex2D::new(0.0, 0.0);
let b = Vertex2D::new(3.0, 4.0);

a.distance_to(b);                          // 5.0
(a + b).x;                                 // 3.0
a.translated(1.0, 2.0);                    // Vertex2D { x: 1, y: 2 }
a.rotated(std::f64::consts::FRAC_PI_2);    // rotate about origin
a.rotated_about(b, 0.0);                   // no-op (pivot == b)

Vertex2D::from((1.0, 2.0));                // From<(f64, f64)>
let (x, y): (f64, f64) = a.into();          // Into<(f64, f64)>
```

### Polygon helpers (free functions)

```rust
let sq = vec![
    Vertex2D::new(0.0, 0.0),
    Vertex2D::new(1.0, 0.0),
    Vertex2D::new(1.0, 1.0),
    Vertex2D::new(0.0, 1.0),
];

polygon_area(&sq);          // signed area (CCW positive)
polygon_area_unsigned(&sq);// 1.0
polygon_perimeter(&sq);    // 4.0
polygon_centroid(&sq);     // (0.5, 0.5)
polygon_bounding_box(&sq); // ((0,0), (1,1))
point_in_polygon(Vertex2D::new(0.5, 0.5), &sq);  // true
point_in_polygon(Vertex2D::new(2.0, 0.5), &sq);  // false
```

### `Trajectory::validate`

NURBS spec rule: `knot_vector.len() == control_points.len() + degree + 1`.

```rust
let traj = Trajectory::linear(vec![
    ControlPoint::new(0.0, 0.0),
    ControlPoint::new(1.0, 0.0),
    ControlPoint::new(1.0, 1.0),
]);
traj.validate()?;  // Ok

let mut bad = traj.clone();
bad.knot_vector = vec![0.0, 0.5];
bad.validate()?;   // Err(KnotVectorMismatch { ... })
```

### Other geometry helpers

| Type | Helpers |
|---|---|
| `NodePosition` | `from_xy_theta(x, y, theta, map_id)`, `with_xy_deviation`, `with_theta_deviation` |
| `NodePositionDeviation` | `new(a, b, theta_rad)` |
| `ControlPoint` | `new(x, y)` (default weight 1.0), `with_weight(w)` |
| `BoundingBox` | `new(length, width)`, `with_height(h)`, `floor_area()`, `volume()` |
| `BoundingBoxReference` | `new(x, y, z)`, `with_theta(theta_rad)` |

---

## Error handling

A single `ValidationError` enum covers every validator:

```rust
pub enum ValidationError {
    Header(HeaderError),
    Timestamp(TimestampError),
    ZoneMissingField { zone_id, field },
    PolygonTooSmall { context, len },
    KnotVectorMismatch { got, expected, control_points, degree },
    InstantActionBlocking { action_id },
    Action(ActionError),
    EmptyOrder,
    OutOfRange { field, value },
}
```

Plus the per-cause types: `HeaderError`, `TimestampError`, `ActionError`,
`UnknownVariant`. They all implement `std::error::Error` and `Display`.

```rust
match zone.validate() {
    Err(ValidationError::ZoneMissingField { zone_id, field }) => {
        eprintln!("zone {zone_id} missing {field}");
    }
    Err(e) => eprintln!("{e}"),
    Ok(())  => println!("valid"),
}
```

---

## Action

```rust
// Instant actions — blocking_type is forced to NONE.
let a = Action::for_instant("pauseOrder", "ia-1");
assert_eq!(a.blocking_type, BlockingType::BlockingNone as i32);
assert!(a.retriable.is_none());

// Order / zone actions — retriable defaults to Some(false).
let a = Action::for_order("pick", "act-pick-1")
    .with_descriptor("Pick pallet")
    .with_blocking_type(BlockingType::BlockingHard)
    .with_parameter("loadId", prost_types::Value { kind: Some(prost_types::value::Kind::StringValue("pallet-42".into())) });

a.validate()?;
```

### `ActionParameter` typed builders

```rust
use ufms::prelude::*;
use ufms::action::ActionParameterExt;

ActionParameter::string("device", "robot-42");    // StringValue
ActionParameter::bool("active", true);            // BoolValue
ActionParameter::number("speed", 1.5);            // NumberValue
ActionParameter::integer("count", 42);            // NumberValue(42.0)
ActionParameter::array("samples", vec![/* … */]);
ActionParameter::object("config", vec![("k".into(), prost_types::Value { … })]);
ActionParameter::null("placeholder");
```

### `ActionBuilder`

Fluent wrapper that holds an `Action` and lets you chain mutations.

---

## Connection

```rust
let conn = Connection::online(header);
assert_eq!(conn.connection_state, ConnectionState::ConnectionOnline as i32);
conn.is_disconnected();    // false

let conn = Connection::broken(header);
assert!(conn.is_disconnected());    // matches BROKEN or OFFLINE
```

The four lifecycle constructors (`online`, `offline`, `hibernating`,
`broken`) match the rules in `connection.proto`'s header comment —
"coordinated shutdown → OFFLINE", "MQTT last-will → BROKEN", etc.

---

## InstantActions

```rust
let msg = InstantActionsBuilder::new(header)
    .add_action(Action::for_instant("pauseOrder", "ia-1"))
    .add_action(Action::for_instant("resumeOrder", "ia-2"))
    .try_build()?;                  // rejects any non-BlockingNone action
```

`InstantActionsBuilder::validate()` enforces the spec rule that every
action in an `InstantActions` message has `blocking_type == NONE`.

---

## Order

```rust
let order = OrderBuilder::new("order-abc")
    .header(header)
    .description("Pick & deliver pallet A1 → B2")
    .node(
        NodeBuilder::new("n1", 0)
            .released(true)
            .descriptor("Pick station A1")
            .position(NodePosition::from_xy_theta(
                1.5, 2.5, Some(1.5708), "warehouse-1",
            ))
            .action(
                Action::for_order("pick", "act-pick-1")
                    .with_blocking_type(BlockingType::BlockingHard)
                    .with_parameter("loadId", /* Value */),
            )
            .build(),
    )
    .edge(
        EdgeBuilder::new("e1", 1)
            .maximum_speed(1.5)
            .maximum_height(2.0)
            .orientation(0.0, OrientationType::OrientationTangential)
            .build(),
    )
    .try_build()?;                  // Err(EmptyOrder) when both lists are empty
```

### `OrderExt`

```rust
order.is_valid_minimal();        // true: at least one node or edge
order.distinct_sequence_ids();   // distinct sequence IDs across nodes+edges
order.validate()?;               // catches EmptyOrder
```

### `Node` / `Edge`

```rust
let n = Node::new("n1", 0)
    .with_descriptor("Pick A1")
    .with_position(NodePosition::from_xy_theta(1.5, 2.5, None, "m1"))
    .add_action(Action::for_order("pick", "a1"));

let e = Edge::new("e1", 1)
    .with_speed_limits(1.5, 2.0, 0.2)        // speed, height, lhd-height cluster
    .with_orientation(0.5, OrientationType::OrientationGlobal, true, Some(1.0))
    .with_corridor(Corridor::new(0.5, 0.5, CorridorReferencePoint::CorridorKinematicCenter));
```

### `Corridor`

```rust
Corridor::new(0.5, 0.5, CorridorReferencePoint::CorridorKinematicCenter)
    .with_release_required(true)
    .with_release_loss_behavior(ReleaseLossBehavior::ReleaseStop);
```

---

## State

```rust
let state = State::new(header)
    .driving(true)
    .paused(false)
    .new_base_request(false)
    .order_id("o1")
    .order_update_id(7)
    .last_node_id("n1")
    .last_node_sequence_id(2)
    .add_load(
        LoadBuilder::new("pallet-42", "EPAL")
            .position("front")
            .weight(450.0)
            .build(),
    )
    .add_error(
        Error::new("batteryLow", ErrorLevel::ErrorWarning,
                   "Battery below 20%", "Return to charging station")
            .add_description_translation("de", "Batterie unter 20%")
            .add_reference("nodeId", "n1"),
    )
    .power_supply(Some(
        PowerSupply::new(78.0, false)
            .with_battery_voltage(48.2)
            .with_battery_health(98.0)
            .with_range(4200.0),
    ))
    .with_operating_mode(OperatingMode::OperatingAutomatic as i32)
    .safety_state(Some(SafetyState::new(EStopType::EstopNone, false)))
    .mobile_robot_position(Some(MobileRobotPosition {
        x: 0.0, y: 0.0, theta: 0.0,
        map_id: "warehouse-1".into(),
        localized: true,
        localization_score: Some(0.95),
        deviation_range: Some(0.02),
    }))
    .velocity(Some(Velocity::new(Some(0.5), Some(0.0), Some(0.0))));
```

### Semantic accessors (`StateExt`)

```rust
state.is_idle();                    // true if both node_states and edge_states empty
state.is_driving();
state.is_paused();
state.order_context();              // OrderContext { order_id, order_update_id, last_node_id, last_node_sequence_id }
state.loads_known();
state.loads();                      // &[Load]
state.errors();                     // &[Error]
state.information();                // &[Info]
state.errors_at_or_above(ErrorLevel::ErrorCritical);   // filter by severity
```

### Other state helpers

| Type | Helpers |
|---|---|
| `Map` | `Map::new(map_id)`, `with_version`, `with_descriptor`, `with_status` |
| `ZoneSetReference` | `new(zone_set_id, map_id)`, `with_status` |
| `NodeState` | `new(node_id, sequence_id)`, `released`, `with_position`, etc. |
| `NodeStatePosition` | `new(x, y, map_id)`, `with_theta` |
| `EdgeState` | `new(edge_id, sequence_id)`, `with_trajectory` |
| `Velocity` | `zero()`, `new(vx, vy, omega)`, `is_known()`, `linear_speed()` |
| `MobileRobotPosition` | `is_trusted()`, `localization_quality()`, `localization_deviation()` |
| `PlannedPath` | `new(trajectory, traversed_nodes)` |
| `PolylinePoint` | `new(x, y, eta)`, `with_theta` |
| `Load` | `new(load_id, load_type)`, `with_weight`, `with_dimensions`, `weight_or_unknown()` |
| `ZoneRequest` | `new(request_id, request_type, zone_id, zone_set_id)` |
| `EdgeRequest` | `new(request_id, request_type, edge_id, sequence_id)` |
| `Error` | `new(error_type, error_level, description, hint)`, `add_reference`, `add_*_translation` |
| `Info` | `new(info_type, info_level)`, `with_descriptor`, `add_reference` |
| `PowerSupply` | `new(state_of_charge, charging)`, `with_battery_voltage`, … |
| `SafetyState` | `new(e_stop_type, field_violation)`, `is_safe_to_drive()` |

---

## Visualization

```rust
let v = Visualization::empty_for_state(header, /* state header id */ 42);
let v = Visualization::new(header, 42)
    .with_planned_path(planned_path)
    .with_velocity(Velocity::new(Some(0.5), Some(0.0), Some(0.0)));
```

---

## Factsheet

`Factsheet` is the largest topic — 7 sub-sections, ~30 nested types. The
ergonomic layer exposes constructors and chainable setters for every one.

### Top-level

```rust
let fs = Factsheet::new("ACME-AMR", "Pallet-moving AMR")
    .with_header(header)
    .with_physical_parameters(PhysicalParameters::new(
        /* min_speed */ 0.1,
        /* max_speed */ 1.5,
        /* max_acceleration */ 1.0,
        /* max_deceleration */ 1.0,
        /* min_height */     0.4,
        /* max_height */     2.0,
        /* width */          0.7,
        /* length */         1.2,
    ).with_angular(0.0, 2.0))
    .with_protocol_limits(ProtocolLimits::new(
        MaximumStringLengths::unbounded(),
        MaximumArrayLengths::unbounded(),
        Timing::new(0.5, 0.1).with_default_state_interval(1.0),
    ))
    .with_mobile_robot_geometry(MobileRobotGeometry::new()
        .add_wheel(WheelDefinition::fixed_wheel(0.3, 0.2, 0.0, 0.15, 0.05).driven_active())
        .add_wheel(WheelDefinition::caster_wheel(-0.3, 0.2, 0.1, 0.05, 0.05))
        .add_envelope_2d(Envelope2D::new("main", vec![
            Vertex2D::new(0.0, 0.0),
            Vertex2D::new(1.2, 0.0),
            Vertex2D::new(1.2, 0.7),
        ])),
    )
    .with_load_specification(LoadSpecification::new()
        .add_load_set(LoadSet::new("DEFAULT", "EPAL",
            BoundingBoxReference::new(0.0, 0.0, 0.0),
            BoundingBox::new(1.2, 0.8).with_height(1.6),
        )
        .with_height_envelope(0.0, 1.6)
        .with_motion_caps(1.5, 1.0, 1.0)
        .with_handling_times(2.0, 2.0))
    )
    .with_mobile_robot_configuration(MobileRobotConfiguration::new()
        .add_version(Version::new("softwareVersion", "v1.03.2"))
        .with_network(Network::new().with_static_address("192.168.1.42", "255.255.255.0", "192.168.1.1"))
        .with_battery_charging(BatteryCharging::new()));
```

### Paired setters

Several factsheet fields are min/max bands or co-occurring pairs. Helpers:

| Type | Helper |
|---|---|
| `PhysicalParameters` | `.with_angular(min, max)` |
| `MaximumStringLengths` | `.with_ids(max_len, numerical_only)` |
| `LoadSet` | `.with_height_envelope(min_m, max_m)` |
| `LoadSet` | `.with_depth_envelope(min_m, max_m)` |
| `LoadSet` | `.with_tilt_envelope(min_rad, max_rad)` |
| `LoadSet` | `.with_motion_caps(max_speed, max_accel, max_decel)` |
| `LoadSet` | `.with_handling_times(pick_s, drop_s)` |
| `Network` | `.with_static_address(ip, netmask, gateway)` |

### Helpers on factsheet sub-types

| Type | Helper |
|---|---|
| `WheelDefinition` | `.fixed_wheel(x, y, theta, d, w)`, `.caster_wheel(x, y, d, w, disp)`, `.mecanum_wheel(x, y, d, w)` |
| `Envelope2D` | `.area()`, `.perimeter()` |
| `Envelope3D` | `.with_url(id, format, url)`, `.with_data(id, format, struct)` (mutually exclusive) |
| `BatteryCharging` | `.validate()` — checks `min ≤ max` on the charging band |

---

## ZoneSet

The most important ergonomic surface — every `ZoneType` gets a dedicated
constructor that enforces the spec's conditional fields.

### Per-zone-type constructors

```rust
Zone::blocked("z", verts);                                                 // ZONE_BLOCKED
Zone::line_guided("z", verts);                                             // ZONE_LINE_GUIDED
Zone::coordinated_replanning("z", verts);                                 // ZONE_COORDINATED_REPLANNING
Zone::release("z", ReleaseLossBehavior::ReleaseStop, verts);               // ZONE_RELEASE
Zone::speed_limit("z", 0.5, verts);                                        // ZONE_SPEED_LIMIT
Zone::action("z", entry, during, exit, verts);                             // ZONE_ACTION
Zone::priority("z", 0.7, verts);                                           // ZONE_PRIORITY
Zone::penalty("z", 0.3, verts);                                            // ZONE_PENALTY
Zone::directed("z", 0.0, DirectedLimitation::DirectedRestricted, verts);  // ZONE_DIRECTED
Zone::bidirected("z", 0.0, BidirectedLimitation::BidirectedSoft, verts);   // ZONE_BIDIRECTED
```

Each constructor populates the **required** conditional fields, so a
`Zone::speed_limit(id, max_speed, verts)` is guaranteed to validate.

### Validation (`Zone::validate`)

Encodes the spec's if/then table for use on decoded messages:

| Zone type | Required field |
|---|---|
| `ZONE_SPEED_LIMIT` | `maximum_speed` |
| `ZONE_RELEASE` | `release_loss_behavior` |
| `ZONE_ACTION` | at least one of `entry_actions` / `during_actions` / `exit_actions` |
| `ZONE_PRIORITY` | `priority_factor` |
| `ZONE_PENALTY` | `penalty_factor` |
| `ZONE_DIRECTED` | `direction` + `directed_limitation` |
| `ZONE_BIDIRECTED` | `bidirected_direction` + `bidirected_limitation` |

Plus common invariants: ≥3 vertices, non-empty `zone_id`.

### Geometric helpers on `Zone`

```rust
zone.contains(Vertex2D::new(0.5, 0.5));   // point-in-polygon
zone.area();                                // signed polygon area
```

### `ZoneSet` and `ZoneSetData`

```rust
let zs = ZoneSetBuilder::new(header, "warehouse-1", "zs-default")
    .descriptor("Default zone set")
    .add_zone(Zone::speed_limit("z-speed-1", 0.5, verts))
    .add_zone(Zone::release("z-rel-1", ReleaseLossBehavior::ReleaseStop, verts))
    .try_build()?;                          // validates every zone

let data = ZoneSetData::new("m", "zs")
    .with_descriptor("Default")
    .add_zone(zone);
```

`ZoneSetExt` provides `validate()` (catches missing payload, empty
`zone_set_id`, and per-zone validation errors) and `zone_count()`.

---

## Responses

```rust
Response::granted("req-1", Some(lease_ts));   // + lease
Response::queued("req-2");
Response::revoked("req-3");
Response::rejected("req-4");

let resp = ResponseBuilder::new(header)
    .response(Response::granted("req-1", None))
    .response(Response::queued("req-2"))
    .build();

resp.lookup("req-1");                  // Some(&Response)
resp.lookup("nope");                   // None

resp.responses[0].has_lease();         // true iff lease_expiry is set
```

---

## Spec-rule validators (recap)

| Spec rule | Where enforced |
|---|---|
| Instant actions have `blocking_type = NONE` | `InstantActionsBuilder::validate` |
| `Order` requires at least one node or edge | `Order::validate` / `OrderBuilder::try_build` |
| `Zone` requires its conditional field(s) per `zone_type` | `Zone::validate` / `ZoneSetExt::validate` |
| Polygons (zones, envelopes) need ≥3 vertices | `Zone::validate` / `Envelope2D::new` invariant |
| NURBS trajectory knot vector length | `Trajectory::validate` |
| `BatteryCharging` band `min ≤ max` | `BatteryCharging::validate` |
| `Header` non-empty `manufacturer` / `serial_number` / `version` (3.x) | `Header::validate` |
| `Action` non-empty `action_type` / `action_id` | `Action::validate` |

Validators are **explicit** (you call `.validate()`) rather than implicit in
constructors, because protobuf types are deserialized from arbitrary bytes
and need to be validatable post-hoc.

---

## Tests

37 integration tests in `tests/models.rs` exercise every constructor and
validator:

```sh
cargo test
```

Coverage:

- `Header::new` round-trip, `Header::validate` negative cases, `HeaderExt`
  on every header-bearing topic
- `Timestamp::to_iso8601_utc` / `from_iso8601_utc` round-trip, fixed
  value `2026-06-29T12:34:56.789Z`
- `Connection` lifecycle tags + `is_disconnected`
- `EnumExt` short-name, `Display`, `FromStr` (3 forms), `is_unspecified`,
  semantic helpers (`accepts_new_orders`, `is_clear`, `is_positive`,
  `is_active`, `is_disconnected`)
- `Vertex2D` arithmetic + distance
- Polygon area / perimeter / centroid / bounding-box / point-in-polygon on
  unit square and triangle
- `BoundingBox::floor_area` / `volume`
- `Trajectory::linear` + `validate` (positive and knot-vector mismatch)
- `Action::for_instant` forces `BlockingNone`, `Action::validate`
  catches empty `action_type` / `action_id`
- `ActionParameter::string` / `bool` / `number` / `integer` typed builders
- `InstantActionsBuilder` rejects non-`BlockingNone`, accepts valid
- `Order::is_valid_minimal`, `Order::validate` (empty), `Order::distinct_sequence_ids`
- `State` semantic helpers (`is_driving`, `is_paused`, `is_idle`,
  `order_context`, `errors_at_or_above`, `loads`)
- `MobileRobotPosition::is_trusted` / `localization_quality` / `localization_deviation`
- `Velocity::zero` / `new` / `is_known` / `linear_speed`
- `SafetyState::is_safe_to_drive` (3 negative cases)
- `Response` constructors + `has_lease` + `Responses::lookup`
- `Zone` per-type validators (positive + missing-field cases)
- `Zone::contains` (point-in-polygon)
- `ZoneSet::validate` + `zone_count`
- `Visualization::empty_for_state` round-trip
- `BatteryCharging::validate` band check (positive + inverted band)

---

## Demo

```sh
cargo run
```

The demo (`src/main.rs`) exercises all 12 ergonomic surfaces, asserts the
key invariants, and prints wire-byte sizes:

```
VDA 5050 v3.0.0 ergonomic models compile and round-trip successfully.
  Connection:     48 bytes wire
  Order:          223 bytes wire
  InstantActions: 106 bytes wire
  State:          318 bytes wire
  Factsheet:      99 bytes wire
  ZoneSet:        226 bytes wire
  Visualization:  50 bytes wire
```

---

## Spec provenance

`VDA5050/` is a vendored copy of the [VDA 5050 v3.0.0 GitHub spec](https://github.com/vda5050).
It contains:

- `proto/*.proto` — 9 input files compiled by `prost-build`
- `json_schemas/*.schema` — matching JSON schemas (one per topic)
- `assets/` — UML / sequence diagrams from the spec
- `LICENSE.txt` — upstream VDA 5050 license
- `README.md` — upstream description and contributing notes

Update by replacing the directory contents and rebuilding.

---

## License

The VDA 5050 spec in `VDA5050/` is licensed under its upstream license
(see `VDA5050/LICENSE.txt`). The Rust code in `src/`, `tests/`, `build.rs`,
and the root `Cargo.toml` is part of the UFMS project and is licensed
separately.