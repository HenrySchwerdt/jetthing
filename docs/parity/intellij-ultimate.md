# IntelliJ IDEA parity: frameworks, databases, and integrations

Framework, database, web, container, and collaboration features. Most need an Ultimate
subscription; per the unified-distribution FAQ, basic database connectivity, SQL support,
the Spring Boot project wizard, and basic Spring/Jakarta EE highlighting are free.

Sources: IntelliJ IDEA public documentation (<https://www.jetbrains.com/help/idea/>), the predefined Windows
keymap reference (<https://www.jetbrains.com/help/idea/reference-keymap-win-default.html>) and the unified-distribution FAQ (<https://lp.jetbrains.com/intellij-idea-unified-faq/>), which lists
the features that need an Ultimate subscription. Rows were written from public
documentation and observable behavior only; no JetBrains code, artwork, or branding
is reused.

Rows tagged `[Ultimate]` require an IntelliJ IDEA Ultimate subscription. Untagged rows are
available in the free distribution. `Weight` (1-5) is the relative importance used by the
weighted dashboard: 5 = core daily workflow, 3 = significant, 2 = moderate, 1 = minor.
Statuses describe Jetthing, conservatively: nothing in this catalog is `Verified`.

| ID | Reference | Feature | Status | Weight | Commands | Tests | Limitations / evidence |
|---|---|---|---|---:|---|---|---|
| IJ-ULT-001 | [spring-initializr-project-wizard](https://www.jetbrains.com/help/idea/spring-initializr-project-wizard.html) | Spring Boot project wizard (Spring Initializr) | Missing | 2 | — | — | free in the unified distribution per FAQ |
| IJ-ULT-002 | [spring-boot](https://www.jetbrains.com/help/idea/spring-boot.html) | Basic Spring syntax highlighting | Missing | 1 | — | — | free in the unified distribution per FAQ |
| IJ-ULT-003 | [spring-projects](https://www.jetbrains.com/help/idea/spring-projects.html) | [Ultimate] Spring bean navigation, autowiring inspections, gutter icons | Missing | 3 | — | — | not started |
| IJ-ULT-004 | [run-debug-configuration-spring-boot](https://www.jetbrains.com/help/idea/run-debug-configuration-spring-boot.html) | [Ultimate] Spring Boot run configuration and actuator endpoints | Missing | 2 | — | — | not started |
| IJ-ULT-005 | [spring-boot](https://www.jetbrains.com/help/idea/spring-boot.html) | [Ultimate] application.properties / application.yml completion | Missing | 2 | — | — | not started |
| IJ-ULT-006 | [spring-projects](https://www.jetbrains.com/help/idea/spring-projects.html) | [Ultimate] Spring Data repository method completion | Missing | 1 | — | — | not started |
| IJ-ULT-007 | [spring-projects](https://www.jetbrains.com/help/idea/spring-projects.html) | [Ultimate] Spring Security support | Missing | 1 | — | — | not started |
| IJ-ULT-008 | [spring-projects](https://www.jetbrains.com/help/idea/spring-projects.html) | [Ultimate] Spring MVC / WebFlux endpoints | Missing | 1 | — | — | not started |
| IJ-ULT-009 | [spring-debugger](https://www.jetbrains.com/help/idea/spring-debugger.html) | [Ultimate] Spring debugger (beans, transactions, properties at runtime) | Missing | 1 | — | — | not started |
| IJ-ULT-010 | [java-ee](https://www.jetbrains.com/help/idea/java-ee.html) | [Ultimate] Jakarta EE (CDI, JAX-RS, Servlets) | Missing | 1 | — | — | not started |
| IJ-ULT-011 | [jakarta-persistence-jpa](https://www.jetbrains.com/help/idea/jakarta-persistence-jpa.html) | [Ultimate] JPA / Hibernate support and JPQL | Missing | 2 | — | — | not started |
| IJ-ULT-012 | [quarkus](https://www.jetbrains.com/help/idea/quarkus.html) | [Ultimate] Quarkus | Missing | 1 | — | — | not started |
| IJ-ULT-013 | [micronaut](https://www.jetbrains.com/help/idea/micronaut.html) | [Ultimate] Micronaut | Missing | 1 | — | — | not started |
| IJ-ULT-014 | [thymeleaf](https://www.jetbrains.com/help/idea/thymeleaf.html) | [Ultimate] Thymeleaf templates beyond basic highlighting | Missing | 1 | — | — | not started |
| IJ-ULT-015 | [application-servers-support](https://www.jetbrains.com/help/idea/application-servers-support.html) | [Ultimate] Application servers (Tomcat, WildFly) run configurations | Missing | 1 | — | — | not started |
| IJ-ULT-016 | [openapi](https://www.jetbrains.com/help/idea/openapi.html) | [Ultimate] OpenAPI specifications editing and preview | Missing | 1 | — | — | not started |
| IJ-ULT-020 | [connecting-to-a-database](https://www.jetbrains.com/help/idea/connecting-to-a-database.html) | Database connections and schema viewing | Missing | 2 | — | — | free in the unified distribution per FAQ |
| IJ-ULT-021 | [relational-databases](https://www.jetbrains.com/help/idea/relational-databases.html) | SQL language support and schema-aware completion | Missing | 2 | — | — | free in the unified distribution per FAQ |
| IJ-ULT-022 | [managing-data-sources](https://www.jetbrains.com/help/idea/managing-data-sources.html) | [Ultimate] Data editor: browse, edit, filter, export table data | Missing | 2 | — | — | not started |
| IJ-ULT-023 | [ddl-data-sources](https://www.jetbrains.com/help/idea/ddl-data-sources.html) | [Ultimate] DDL data sources and schema comparison | Missing | 1 | — | — | not started |
| IJ-ULT-024 | [mongodb](https://www.jetbrains.com/help/idea/mongodb.html) | [Ultimate] NoSQL databases (MongoDB, Redis) | Missing | 1 | — | — | not started |
| IJ-ULT-025 | [query-execution-plan](https://www.jetbrains.com/help/idea/query-execution-plan.html) | [Ultimate] Query execution plans | Missing | 1 | — | — | not started |
| IJ-ULT-026 | [diagrams](https://www.jetbrains.com/help/idea/diagrams.html) | [Ultimate] Database diagrams | Missing | 1 | — | — | not started |
| IJ-ULT-030 | [http-client-in-product-code-editor](https://www.jetbrains.com/help/idea/http-client-in-product-code-editor.html) | [Ultimate] HTTP Client: .http request files with run gutter | Missing | 2 | — | — | not started |
| IJ-ULT-031 | [http-client-variables](https://www.jetbrains.com/help/idea/http-client-variables.html) | [Ultimate] HTTP Client environments and variables | Missing | 1 | — | — | not started |
| IJ-ULT-032 | [http-response-handling-api-reference](https://www.jetbrains.com/help/idea/http-response-handling-api-reference.html) | [Ultimate] HTTP Client response handler scripts and tests | Missing | 1 | — | — | not started |
| IJ-ULT-033 | [http-client-cli](https://www.jetbrains.com/help/idea/http-client-cli.html) | [Ultimate] HTTP Client CLI | Missing | 1 | — | — | not started |
| IJ-ULT-034 | [javascript-specific-guidelines](https://www.jetbrains.com/help/idea/javascript-specific-guidelines.html) | [Ultimate] JavaScript language support | Missing | 2 | — | — | not started |
| IJ-ULT-035 | [typescript-support](https://www.jetbrains.com/help/idea/typescript-support.html) | [Ultimate] TypeScript language support | Missing | 2 | — | — | not started |
| IJ-ULT-036 | [web-development](https://www.jetbrains.com/help/idea/web-development.html) | [Ultimate] Web frameworks (React, Angular, Vue) and CSS | Missing | 1 | — | — | not started |
| IJ-ULT-037 | [npm](https://www.jetbrains.com/help/idea/npm.html) | [Ultimate] npm / package.json scripts integration | Partial | 1 | `build.project` | — | npm detected as a build plan only |
| IJ-ULT-040 | [docker](https://www.jetbrains.com/help/idea/docker.html) | Docker: images, containers, Dockerfile run configurations | Missing | 2 | — | — | not started |
| IJ-ULT-041 | [kubernetes](https://www.jetbrains.com/help/idea/kubernetes.html) | [Ultimate] Kubernetes: clusters, resources, Helm | Missing | 1 | — | — | not started |
| IJ-ULT-042 | [dev-containers-starting-page](https://www.jetbrains.com/help/idea/dev-containers-starting-page.html) | [Ultimate] Dev Containers | Missing | 1 | — | — | not started |
| IJ-ULT-043 | [remote](https://www.jetbrains.com/help/idea/remote.html) | Remote development (backend on remote host) | Missing | 1 | — | — | not started |
| IJ-ULT-044 | [deploying-applications](https://www.jetbrains.com/help/idea/deploying-applications.html) | [Ultimate] Deployment to remote servers (SFTP/FTP) | Missing | 1 | — | — | not started |
| IJ-ULT-045 | [code-with-me](https://www.jetbrains.com/help/idea/code-with-me.html) | Code With Me (collaborative editing) | Missing | 1 | — | — | not started |
| IJ-ULT-050 | [dsm-analysis](https://www.jetbrains.com/help/idea/dsm-analysis.html) | [Ultimate] Dependency structure matrix | Missing | 1 | — | — | not started |
| IJ-ULT-051 | [project-module-dependencies-diagram](https://www.jetbrains.com/help/idea/project-module-dependencies-diagram.html) | [Ultimate] Module dependency diagrams | Missing | 1 | — | — | not started |
| IJ-ULT-052 | [ai-assistant-in-jetbrains-ides](https://www.jetbrains.com/help/idea/ai-assistant-in-jetbrains-ides.html) | [Ultimate] AI assistance (chat, commit messages, explain code) | Missing | 2 | — | — | not started |
| IJ-ULT-053 | [jupyter-notebook-support](https://www.jetbrains.com/help/idea/jupyter-notebook-support.html) | [Ultimate] Jupyter notebooks | Missing | 1 | — | — | not started |
| IJ-ULT-054 | [big-data-tools-support](https://www.jetbrains.com/help/idea/big-data-tools-support.html) | [Ultimate] Big Data Tools (Spark, Kafka) | Missing | 1 | — | — | not started |
| IJ-ULT-055 | [wiremock](https://www.jetbrains.com/help/idea/wiremock.html) | [Ultimate] WireMock stubs support | Missing | 1 | — | — | not started |
