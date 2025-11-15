# CSV Schema v3 - Enhanced with Health Monitoring and Repair Capabilities

This document extends CSV Schema v2 with additional fields for comprehensive health monitoring, repair tracking, and operational excellence within the NOA deployment framework.

## Mandatory Fields (from v1)

1. **agent_name** - Unique identifier for the agent
2. **role** - Primary function and responsibility
3. **layer** - Hierarchical layer (cecca, board, executive, stack_chief, specialist, micro)
4. **scope** - Operational scope and boundaries
5. **tools** - Available tools and capabilities
6. **inputs** - Expected input types and formats
7. **outputs** - Generated output types and formats
8. **guardrails** - Safety and operational constraints
9. **escalation_to** - Escalation path for issues
10. **stack** - Associated stack identifier

## Extended Metadata Fields (from v2)

11. **actions** - Available actions and operations
12. **agent_code** - Internal agent code identifier
13. **agent_id** - System-wide unique identifier
14. **aka** - Alternative names or aliases
15. **artifacts** - Associated artifacts and resources
16. **audit_logging** - Audit and logging configuration
17. **auto_remediation** - Automatic remediation capabilities
18. **autonomy_level** - Level of autonomous operation
19. **budget_cap** - Resource and cost limitations
20. **cache** - Caching configuration and strategy
21. **capabilities** - Detailed capability specifications
22. **capability_pack** - Associated capability package
23. **capability_pack_id** - Capability package identifier
24. **category** - Functional category classification
25. **code_paths** - Code execution paths
26. **commands** - Available command set
27. **connectors** - Integration connectors
28. **cost_center** - Financial cost center assignment
29. **court_gate** - Governance gate configuration
30. **court_policy_id** - Associated policy identifier
31. **created_at** - Creation timestamp
32. **data_sources** - Associated data sources
33. **dependencies** - System dependencies
34. **depends_on_agents** - Agent dependencies
35. **display_name** - Human-readable display name
36. **efg_requirements** - Environment, framework, and governance requirements
37. **embedding_models** - Associated embedding models
38. **endpoints** - API endpoints and interfaces
39. **epic** - Associated epic or project
40. **escalation_policy** - Detailed escalation policy
41. **export_control** - Export control classification
42. **failure_modes** - Known failure modes
43. **governance_role** - Governance and compliance role
44. **human_approval_required** - Human approval requirements
45. **last_updated** - Last update timestamp
46. **license_category** - License classification
47. **manifests** - Associated manifest files
48. **maturity_level** - Development maturity level
49. **memory_scope** - Memory usage scope
50. **metrics** - Performance metrics configuration
51. **models** - Associated AI/ML models
52. **operational_readiness_score** - Readiness assessment score
53. **operations_domain** - Operational domain classification
54. **owner_role** - Ownership role assignment
55. **pii_handling** - Personal information handling policy
56. **plane** - Operational plane (control, data, management)
57. **policies** - Associated policies
58. **provides_capabilities** - Capabilities provided to other agents
59. **purpose** - Detailed purpose description
60. **raci_a** - RACI matrix - Accountable
61. **raci_c** - RACI matrix - Consulted
62. **raci_i** - RACI matrix - Informed
63. **raci_r** - RACI matrix - Responsible
64. **risk_class** - Risk classification level
65. **scheduler_owner** - Scheduler ownership
66. **scheduling** - Scheduling configuration
67. **security_level** - Security classification level
68. **sla** - Service level agreement
69. **source_file** - Source file reference
70. **source_row_index** - Source row index
71. **source_rows_json** - Source rows in JSON format
72. **spawn_policy** - Agent spawning policy
73. **state_store** - State storage configuration
74. **status** - Current operational status
75. **subcategory** - Functional subcategory
76. **telemetry** - Telemetry configuration
77. **telemetry_topic** - Telemetry topic assignment
78. **tools_stack** - Tools stack configuration
79. **triggers** - Event triggers
80. **type** - Agent type classification
81. **version** - Agent version

## New Health Monitoring and Repair Fields (v3)

82. **health_status** - Current health assessment
    - Values: "Healthy", "Needs Repair", "Critical Issues", "Unknown"
    - Description: Overall health assessment based on comprehensive analysis

83. **repair_recommendations** - Specific repair recommendations
    - Format: Detailed text with actionable recommendations
    - Description: Specific steps to improve agent health and functionality

84. **issues_identified** - Identified issues and problems
    - Format: Detailed description of issues found
    - Description: Comprehensive list of issues requiring attention

## Health Status Definitions

### Healthy
- Agent is fully functional and operating within expected parameters
- No critical issues identified
- All dependencies and integrations working properly
- Performance metrics within acceptable ranges

### Needs Repair
- Agent has identified issues that require attention
- Functionality may be impacted but not critically compromised
- Repair recommendations available for improvement
- Proactive maintenance required

### Critical Issues
- Agent has severe problems affecting core functionality
- Immediate attention required to prevent system impact
- May require emergency intervention
- High priority for repair operations

### Unknown
- Health status could not be determined
- Insufficient data for assessment
- Requires further analysis
- Default status for unanalyzed agents

## Validation Rules

1. All mandatory fields (1-10) must be present and non-empty
2. **health_status** must be one of the defined values
3. **repair_recommendations** should be provided for agents with "Needs Repair" or "Critical Issues" status
4. **issues_identified** should detail specific problems found
5. Timestamps should follow ISO 8601 format
6. All text fields should be properly escaped for CSV format

## Usage Guidelines

1. Use this schema for all new NOA deployment kits
2. Existing v2 files can be upgraded by adding the new health monitoring fields
3. Regular health assessments should update the health monitoring fields
4. Repair recommendations should be actionable and specific
5. Issues should be clearly documented with sufficient detail for resolution

## Compatibility

- Backward compatible with v1 and v2 schemas
- New fields are additive and do not modify existing field definitions
- Tools supporting v2 can ignore v3 fields without impact
- Full v3 support provides enhanced health monitoring capabilities

---

*Schema Version: 3.0*
*Last Updated: $(date)*
*Compatibility: NOA Deployment Kit v3.1+ Extended with Health Monitoring*

