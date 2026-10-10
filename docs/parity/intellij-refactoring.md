# IntelliJ IDEA parity: refactorings

Refactorings from the Refactor menu and Refactor This popup (Ctrl+Alt+Shift+T). Each
refactoring is expected to preview usages, detect conflicts, and be undoable as one
project-wide action.

Sources: IntelliJ IDEA public documentation (<https://www.jetbrains.com/help/idea/>), the predefined Windows
keymap reference (<https://www.jetbrains.com/help/idea/reference-keymap-win-default.html>) and the unified-distribution FAQ (<https://lp.jetbrains.com/intellij-idea-unified-faq/>), which lists
the features that need an Ultimate subscription. Rows were written from public
documentation and observable behavior only; no JetBrains code, artwork, or branding
is reused.

Rows tagged `[Ultimate]` require an IntelliJ IDEA Ultimate subscription. Untagged rows are
available in the free distribution. `Weight` (1-5) is the relative importance used by the
weighted dashboard: 5 = core daily workflow, 3 = significant, 2 = moderate, 1 = minor.
Statuses describe Jetthing, conservatively: nothing in this catalog is `Verified`.

| ID | Reference | Feature | Status | Weight | Spec | Commands | Tests | Limitations / evidence |
|---|---|---|---|---:|---|---|---|---|
| IJ-REF-001 | [refactoring-source-code](https://www.jetbrains.com/help/idea/refactoring-source-code.html) | Refactor This popup (Ctrl+Alt+Shift+T) | Missing | 3 | `PLATFORM-EDIT-135`, `JAVA-REF-001` | — | — | not started |
| IJ-REF-002 | [refactoring-source-code](https://www.jetbrains.com/help/idea/refactoring-source-code.html) | Refactoring preview and conflict dialogs | Missing | 3 | `JAVA-REF-045` | — | — | not started |
| IJ-REF-003 | [refactoring-source-code](https://www.jetbrains.com/help/idea/refactoring-source-code.html) | Project-wide undo of a refactoring | Missing | 3 | — | — | — | not started |
| IJ-REF-010 | [rename-refactorings](https://www.jetbrains.com/help/idea/rename-refactorings.html) | Rename symbol (Shift+F6) in place | Missing | 5 | `PLATFORM-EDIT-134`, `JAVA-REF-002` | — | — | LSP rename not yet wired |
| IJ-REF-011 | [rename-refactorings](https://www.jetbrains.com/help/idea/rename-refactorings.html) | Rename file / class / package with references | Partial | 3 | `JAVA-REF-003` | `project.rename` | `file_operations_keep_editors_and_tree_consistent` | renames files/directories (Shift+F6 in the tree) and keeps open editors; no reference updates |
| IJ-REF-012 | [rename-refactorings](https://www.jetbrains.com/help/idea/rename-refactorings.html) | Rename in comments and strings, related symbols (tests, getters) | Missing | 1 | `JAVA-REF-046`, `JAVA-REF-047` | — | — | not started |
| IJ-REF-020 | [change-signature](https://www.jetbrains.com/help/idea/change-signature.html) | Change Signature (Ctrl+F6) | Missing | 3 | `JAVA-REF-004` | — | — | not started |
| IJ-REF-021 | [extract-method](https://www.jetbrains.com/help/idea/extract-method.html) | Extract Method (Ctrl+Alt+M) | Missing | 5 | `JAVA-REF-013` | — | — | not started |
| IJ-REF-022 | [extract-variable](https://www.jetbrains.com/help/idea/extract-variable.html) | Introduce Variable (Ctrl+Alt+V) | Missing | 5 | `JAVA-REF-006` | — | — | not started |
| IJ-REF-023 | [extract-constant](https://www.jetbrains.com/help/idea/extract-constant.html) | Introduce Constant (Ctrl+Alt+C) | Missing | 3 | `JAVA-REF-007` | — | — | not started |
| IJ-REF-024 | [extract-field](https://www.jetbrains.com/help/idea/extract-field.html) | Introduce Field (Ctrl+Alt+F) | Missing | 3 | `JAVA-REF-008` | — | — | not started |
| IJ-REF-025 | [extract-parameter](https://www.jetbrains.com/help/idea/extract-parameter.html) | Introduce Parameter (Ctrl+Alt+P) | Missing | 3 | `JAVA-REF-009` | — | — | not started |
| IJ-REF-026 | [extract-parameter](https://www.jetbrains.com/help/idea/extract-parameter.html) | Introduce Functional Parameter | Missing | 1 | `JAVA-REF-010` | — | — | not started |
| IJ-REF-027 | [extract-into-class-refactorings](https://www.jetbrains.com/help/idea/extract-into-class-refactorings.html) | Introduce Parameter Object | Missing | 1 | `JAVA-REF-012` | — | — | not started |
| IJ-REF-028 | [extract-interface](https://www.jetbrains.com/help/idea/extract-interface.html) | Extract Interface | Missing | 2 | `JAVA-REF-016` | — | — | not started |
| IJ-REF-029 | [extract-superclass](https://www.jetbrains.com/help/idea/extract-superclass.html) | Extract Superclass | Missing | 2 | `JAVA-REF-017` | — | — | not started |
| IJ-REF-030 | [extract-into-class-refactorings](https://www.jetbrains.com/help/idea/extract-into-class-refactorings.html) | Extract Delegate / Extract Class | Missing | 1 | `JAVA-REF-015` | — | — | not started |
| IJ-REF-031 | [extract-into-class-refactorings](https://www.jetbrains.com/help/idea/extract-into-class-refactorings.html) | Extract Method Object | Missing | 1 | `JAVA-REF-014` | — | — | not started |
| IJ-REF-040 | [inline](https://www.jetbrains.com/help/idea/inline.html) | Inline variable / method / constant (Ctrl+Alt+N) | Missing | 5 | `JAVA-REF-018` | — | — | not started |
| IJ-REF-041 | [inline](https://www.jetbrains.com/help/idea/inline.html) | Inline superclass / class | Missing | 1 | — | — | — | not started |
| IJ-REF-050 | [move-refactorings](https://www.jetbrains.com/help/idea/move-refactorings.html) | Move class / file / package (F6) | Missing | 3 | `JAVA-REF-019` | — | — | not started |
| IJ-REF-051 | [move-refactorings](https://www.jetbrains.com/help/idea/move-refactorings.html) | Move static members / instance method | Missing | 2 | `JAVA-REF-020`, `JAVA-REF-021` | — | — | not started |
| IJ-REF-052 | [move-refactorings](https://www.jetbrains.com/help/idea/move-refactorings.html) | Copy class / file (F5) | Missing | 2 | `JAVA-REF-023` | — | — | not started |
| IJ-REF-053 | [pull-members-up](https://www.jetbrains.com/help/idea/pull-members-up.html) | Pull Members Up | Missing | 2 | `JAVA-REF-025` | — | — | not started |
| IJ-REF-054 | [push-members-down-dialog](https://www.jetbrains.com/help/idea/push-members-down-dialog.html) | Push Members Down | Missing | 1 | `JAVA-REF-026` | — | — | not started |
| IJ-REF-060 | [safe-delete](https://www.jetbrains.com/help/idea/safe-delete.html) | Safe Delete (Alt+Delete) with usage search | Missing | 3 | `JAVA-REF-024` | — | — | not started |
| IJ-REF-061 | [type-migration](https://www.jetbrains.com/help/idea/type-migration.html) | Type Migration (Ctrl+Shift+F6) | Missing | 1 | `JAVA-REF-005` | — | — | not started |
| IJ-REF-062 | [encapsulate-fields](https://www.jetbrains.com/help/idea/encapsulate-fields.html) | Encapsulate Fields | Missing | 1 | `JAVA-REF-029` | — | — | not started |
| IJ-REF-063 | [convert-anonymous-to-inner](https://www.jetbrains.com/help/idea/convert-anonymous-to-inner.html) | Convert Anonymous to Inner | Missing | 1 | `JAVA-REF-042`, `JAVA-REF-022` | — | — | not started |
| IJ-REF-064 | [convert-to-instance-method](https://www.jetbrains.com/help/idea/convert-to-instance-method.html) | Convert to Instance Method | Missing | 1 | `JAVA-REF-030` | — | — | not started |
| IJ-REF-065 | [make-method-static](https://www.jetbrains.com/help/idea/make-method-static.html) | Make Static | Missing | 1 | `JAVA-REF-031` | — | — | not started |
| IJ-REF-066 | [invert-boolean-refactoring](https://www.jetbrains.com/help/idea/invert-boolean-refactoring.html) | Invert Boolean | Missing | 1 | `JAVA-REF-032` | — | — | not started |
| IJ-REF-067 | [replace-constructor-with-builder](https://www.jetbrains.com/help/idea/replace-constructor-with-builder.html) | Replace Constructor with Builder | Missing | 1 | `JAVA-REF-034` | — | — | not started |
| IJ-REF-068 | [replace-constructor-with-factory-method](https://www.jetbrains.com/help/idea/replace-constructor-with-factory-method.html) | Replace Constructor with Factory Method | Missing | 1 | `JAVA-REF-033` | — | — | not started |
| IJ-REF-069 | [replace-inheritance-with-delegation](https://www.jetbrains.com/help/idea/replace-inheritance-with-delegation.html) | Replace Inheritance with Delegation | Missing | 1 | `JAVA-REF-028` | — | — | not started |
| IJ-REF-070 | [use-interface-where-possible](https://www.jetbrains.com/help/idea/use-interface-where-possible.html) | Use Interface Where Possible | Missing | 1 | `JAVA-REF-027` | — | — | not started |
| IJ-REF-071 | [remove-middleman](https://www.jetbrains.com/help/idea/remove-middleman.html) | Remove Middleman | Missing | 1 | `JAVA-REF-037` | — | — | not started |
| IJ-REF-072 | [wrap-return-value](https://www.jetbrains.com/help/idea/wrap-return-value.html) | Wrap Method Return Value | Missing | 1 | `JAVA-REF-038` | — | — | not started |
| IJ-REF-073 | [generify-refactoring](https://www.jetbrains.com/help/idea/generify-refactoring.html) | Generify | Missing | 1 | `JAVA-REF-035` | — | — | not started |
| IJ-REF-074 | [migrate](https://www.jetbrains.com/help/idea/migrate.html) | Migrate (package/class migration maps) | Missing | 1 | `JAVA-REF-036` | — | — | not started |
| IJ-REF-075 | [find-and-replace-code-duplicates](https://www.jetbrains.com/help/idea/find-and-replace-code-duplicates.html) | Find and Replace Code Duplicates | Missing | 1 | `JAVA-REF-040` | — | — | not started |
| IJ-REF-076 | [maven-refactorings](https://www.jetbrains.com/help/idea/maven-refactorings.html) | Maven refactorings (extract managed dependency, introduce property) | Missing | 1 | — | — | — | not started |
