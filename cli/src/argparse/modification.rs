use super::args::{any, arg_matching, depends_colon, minus_tag, plus_tag, wait_colon, TaskId};
use super::ArgList;
use crate::usage;
use nom::{branch::alt, combinator::*, multi::fold_many0, IResult};
use std::collections::HashSet;
use taskchampion::chrono::prelude::*;
use taskchampion::{Status, Tag};

#[derive(Debug, PartialEq, Clone)]
pub enum DescriptionMod {
    /// do not change the description
    None,

    /// Prepend the given value to the description, with a space separator
    Prepend(String),

    /// Append the given value to the description, with a space separator
    Append(String),

    /// Set the description
    Set(String),
}

impl Default for DescriptionMod {
    fn default() -> Self {
        Self::None
    }
}

/// A modification represents a change to a task: adding or removing tags, setting the
/// description, and so on.
#[derive(Debug, PartialEq, Clone, Default)]
pub(crate) struct Modification {
    /// Change the description
    pub(crate) description: DescriptionMod,

    /// Set the status
    pub(crate) status: Option<Status>,

    /// Set (or, with `Some(None)`, clear) the wait timestamp
    pub(crate) wait: Option<Option<DateTime<Utc>>>,

    /// Set the "active" state, that is, start (true) or stop (false) the task.
    pub(crate) active: Option<bool>,

    /// Add tags
    pub(crate) add_tags: HashSet<Tag>,

    /// Remove tags
    pub(crate) remove_tags: HashSet<Tag>,

    /// Add dependencies
    pub(crate) add_dependencies: HashSet<TaskId>,

    /// Remove dependencies
    pub(crate) remove_dependencies: HashSet<TaskId>,

    /// Add annotation
    pub(crate) annotate: Option<String>,
}

/// A single argument that is part of a modification, used internally to this module
enum ModArg<'a> {
    Description(&'a str),
    PlusTag(Tag),
    MinusTag(Tag),
    Wait(Option<DateTime<Utc>>),
    AddDependencies(Vec<TaskId>),
    RemoveDependencies(Vec<TaskId>),
}

impl Modification {
    pub(super) fn parse(input: ArgList) -> IResult<ArgList, Modification> {
        fn fold(mut acc: Modification, mod_arg: ModArg) -> Modification {
            match mod_arg {
                ModArg::Description(description) => {
                    if let DescriptionMod::Set(existing) = acc.description {
                        acc.description =
                            DescriptionMod::Set(format!("{} {}", existing, description));
                    } else {
                        acc.description = DescriptionMod::Set(description.to_string());
                    }
                }
                ModArg::PlusTag(tag) => {
                    acc.add_tags.insert(tag);
                }
                ModArg::MinusTag(tag) => {
                    acc.remove_tags.insert(tag);
                }
                ModArg::Wait(wait) => {
                    acc.wait = Some(wait);
                }
                ModArg::AddDependencies(task_ids) => {
                    for tid in task_ids {
                        acc.add_dependencies.insert(tid);
                    }
                }
                ModArg::RemoveDependencies(task_ids) => {
                    for tid in task_ids {
                        acc.remove_dependencies.insert(tid);
                    }
                }
            }
            acc
        }
        fold_many0(
            alt((
                Self::plus_tag,
                Self::minus_tag,
                Self::wait,
                Self::dependencies,
                // this must come last
                Self::description,
            )),
            Modification {
                ..Default::default()
            },
            fold,
        )(input)
    }

    fn description(input: ArgList) -> IResult<ArgList, ModArg> {
        fn to_modarg(input: &str) -> Result<ModArg, ()> {
            Ok(ModArg::Description(input))
        }
        map_res(arg_matching(any), to_modarg)(input)
    }

    fn plus_tag(input: ArgList) -> IResult<ArgList, ModArg> {
        fn to_modarg(input: Tag) -> Result<ModArg<'static>, ()> {
            Ok(ModArg::PlusTag(input))
        }
        map_res(arg_matching(plus_tag), to_modarg)(input)
    }

    fn minus_tag(input: ArgList) -> IResult<ArgList, ModArg> {
        fn to_modarg(input: Tag) -> Result<ModArg<'static>, ()> {
            Ok(ModArg::MinusTag(input))
        }
        map_res(arg_matching(minus_tag), to_modarg)(input)
    }

    fn wait(input: ArgList) -> IResult<ArgList, ModArg> {
        fn to_modarg(input: Option<DateTime<Utc>>) -> Result<ModArg<'static>, ()> {
            Ok(ModArg::Wait(input))
        }
        map_res(arg_matching(wait_colon), to_modarg)(input)
    }

    fn dependencies(input: ArgList) -> IResult<ArgList, ModArg> {
        fn to_modarg(input: (bool, Vec<TaskId>)) -> Result<ModArg<'static>, ()> {
            Ok(if input.0 {
                ModArg::AddDependencies(input.1)
            } else {
                ModArg::RemoveDependencies(input.1)
            })
        }
        map_res(arg_matching(depends_colon), to_modarg)(input)
    }

    pub(super) fn get_usage(u: &mut usage::Usage) {
        u.modifications.push(usage::Modification {
            syntax: "DESCRIPTION",
            summary: "Set description/annotation",
            description: "
                Set the task description (or the task annotation for `ta annotate`).  Multiple
                arguments are combined into a single space-separated description.  To avoid
                surprises from shell quoting, prefer to use a single quoted argument, for example
                `ta 19 modify \"return library books\"`",
        });
        u.modifications.push(usage::Modification {
            syntax: "+TAG",
            summary: "Tag task",
            description: "Add the given tag to the task.",
        });
        u.modifications.push(usage::Modification {
            syntax: "-TAG",
            summary: "Un-tag task",
            description: "Remove the given tag from the task.",
        });
        u.modifications.push(usage::Modification {
            syntax: "status:{pending,completed,deleted}",
            summary: "Set the task's status",
            description: "Set the status of the task explicitly.",
        });
        u.modifications.push(usage::Modification {
            syntax: "wait:<timestamp>",
            summary: "Set or unset the task's wait time",
            description: "
                Set the time before which the task is not actionable and should not be shown in
                reports, e.g., `wait:3day` to wait for three days.  With `wait:`, the time is
                un-set.  See the documentation for the timestamp syntax.",
        });
        u.modifications.push(usage::Modification {
            syntax: "depends:<task-list>",
            summary: "Add task dependencies",
            description: "
                Add a dependency of this task on the given tasks.  The tasks can be specified
                in the same syntax as for filters, e.g., `depends:13,94500c95`.",
        });
        u.modifications.push(usage::Modification {
            syntax: "depends:-<task-list>",
            summary: "Remove task dependencies",
            description: "
                Remove the dependency of this task on the given tasks.",
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::argparse::NOW;
    use pretty_assertions::assert_eq;
    use taskchampion::chrono::Duration;

    #[test]
    fn test_empty() {
        let (input, modification) = Modification::parse(argv![]).unwrap();
        assert_eq!(input.len(), 0);
        assert_eq!(
            modification,
            Modification {
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_single_arg_description() {
        let (input, modification) = Modification::parse(argv!["newdesc"]).unwrap();
        assert_eq!(input.len(), 0);
        assert_eq!(
            modification,
            Modification {
                description: DescriptionMod::Set(s!("newdesc")),
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_add_tags() {
        let (input, modification) = Modification::parse(argv!["+abc", "+def"]).unwrap();
        assert_eq!(input.len(), 0);
        assert_eq!(
            modification,
            Modification {
                add_tags: set![tag!("abc"), tag!("def")],
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_set_wait() {
        let (input, modification) = Modification::parse(argv!["wait:2d"]).unwrap();
        assert_eq!(input.len(), 0);
        assert_eq!(
            modification,
            Modification {
                wait: Some(Some(*NOW + Duration::days(2))),
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_add_deps() {
        let (input, modification) = Modification::parse(argv!["depends:13,e72b73d1-9e88"]).unwrap();
        assert_eq!(input.len(), 0);
        let mut deps = HashSet::new();
        deps.insert(TaskId::WorkingSetId(13));
        deps.insert(TaskId::PartialUuid("e72b73d1-9e88".into()));
        assert_eq!(
            modification,
            Modification {
                add_dependencies: deps,
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_remove_deps() {
        let (input, modification) =
            Modification::parse(argv!["depends:-13,e72b73d1-9e88"]).unwrap();
        assert_eq!(input.len(), 0);
        let mut deps = HashSet::new();
        deps.insert(TaskId::WorkingSetId(13));
        deps.insert(TaskId::PartialUuid("e72b73d1-9e88".into()));
        assert_eq!(
            modification,
            Modification {
                remove_dependencies: deps,
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_unset_wait() {
        let (input, modification) = Modification::parse(argv!["wait:"]).unwrap();
        assert_eq!(input.len(), 0);
        assert_eq!(
            modification,
            Modification {
                wait: Some(None),
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_multi_arg_description() {
        let (input, modification) = Modification::parse(argv!["new", "desc", "fun"]).unwrap();
        assert_eq!(input.len(), 0);
        assert_eq!(
            modification,
            Modification {
                description: DescriptionMod::Set(s!("new desc fun")),
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_multi_arg_description_and_tags() {
        let (input, modification) =
            Modification::parse(argv!["new", "+next", "desc", "-daytime", "fun"]).unwrap();
        assert_eq!(input.len(), 0);
        assert_eq!(
            modification,
            Modification {
                description: DescriptionMod::Set(s!("new desc fun")),
                add_tags: set![tag!("next")],
                remove_tags: set![tag!("daytime")],
                ..Default::default()
            }
        );
    }
}
