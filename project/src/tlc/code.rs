//! TLC error codes (*ec*).
//!
//! So called *error codes* are not error codes at all (in general). They are just codes specifying
//! a kind of message produced by TLC. Some of them do correspond to errors.
//!
//! Reference: [`EC.java` on the TLA+ repo][ref]
//!
//! [ref]: https://github.com/tlaplus/tlaplus/blob/a7ff6bece9dd2f2592126a14c5fbc61cb3cf4ac1/tlatools/org.lamport.tlatools/src/tlc2/output/EC.java

prelude!();

/// Type-safe wrapper for message codes ([`isize`]).
///
/// **Constructing** a code is illegal outside of this module, hence the private constructor.
/// **Accessing** the code however is fine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Code {
    code: isize,
}
impl Code {
    /// Private constructor.
    fn new(code: isize) -> Code {
        Self { code }
    }

    /// Code accessor.
    pub fn get(&self) -> isize {
        self.code
    }
}
implem! {
    for Code {
        Display {
            |&self, fmt| write!(fmt, "#({})", self.code)
        }
        Into<isize> {
            |self| self.code
        }
    }
    impl('a) for &'a Code {
        Into<isize> {
            |self| self.code
        }
    }
}

macro_rules! code_variant_construct {
    ($contents:expr, $desc:expr, $variant:expr,
        |$code_contents_arg:ident| $code_new:expr
    ) => {{
        let work = || -> Res<Self> {
            let $code_contents_arg = $contents;
            $code_new
        };
        let mut res = work();
        if res.is_err() {
            for elm in $contents.iter().rev() {
                res = res.context(anyhow!("`{}`", elm))
            }
            res = res
                .context("input from TLC:")
                .context(anyhow!("failed to construct TLC message `{}`", $desc));
        }
        res.map(Some)
    }};
    ($contents:expr, $desc:expr, $variant:expr $(,)?) => {
        Ok(Some($variant))
    };
}

/// Defines error codes.
macro_rules! code_enums {
    {
        type Code = $int_ty:ty;
        unique name = $unique_name:ident;

        $(
            $(#[$enum_meta:meta])*
            $enum_vis:vis enum $enum_name:ident {
                $(codes {
                    $(
                        $(#[$code_variant_meta:meta])*
                        $code_variant:ident = ($code:expr, $code_desc:literal)
                        $({ $($code_fields:tt)* } =>
                            |$code_contents_arg:ident| $code_new:expr
                        )? ,
                    )*
                })?
                $(subs {
                    $(
                        $(#[$sub_code_variant_meta:meta])*
                        $sub_code_variant:ident($sub_enum:ident) ,
                    )*
                })?
            }
            $(impl $_enum_name:ident { $($impl_body:tt)* })?
        )+
    } => {
        /// Dead code. This is just used to make sure all codes are distinct. If they're not, a
        /// warning will be issued below saying that one pattern is unreachable.
        #[allow(dead_code)]
        fn $unique_name(input: $int_ty) {
            match input {
                $($($(
                    $code => (),
                )*)?)+
                _ => (),
            }
        }

        $(
            $(#[$enum_meta])*
            #[derive(Debug, Clone)]
            $enum_vis enum $enum_name {
                $( $(
                    $(#[$code_variant_meta])*
                    $code_variant $({$($code_fields)*})?,
                )* )?
                $( $(
                    $(#[$sub_code_variant_meta])*
                    $sub_code_variant($sub_enum),
                )* )?
            }
            impl $enum_name {
                /// Constructor from a code.
                pub fn from_code(
                    code: impl Into<$int_ty>,
                    _contents: &tlc::msg::Elms,
                ) -> Res<Option<Self>> {
                    let code = code.into();
                    match code {
                        $($(
                            $code => code_variant_construct!(
                                _contents, $code_desc, Self::$code_variant,
                                $(|$code_contents_arg| $code_new)?
                            ),
                        )*)?
                        _ => {
                            $($(
                                let opt = $sub_enum::from_code(code, _contents)?
                                    .map(Self::$sub_code_variant);
                                if opt.is_some() {
                                    return Ok(opt)
                                }
                            )*)?
                            Ok(None)
                        }
                    }
                }

                /// Code value accessor.
                pub fn code(&self) -> Code {
                    match self {
                        $( $(
                            Self::$code_variant { .. } => Code::new($code),
                        )* )?
                        $( $(
                            Self::$sub_code_variant(sub) => sub.code(),
                        )* )?
                    }
                }

                /// Description of a code.
                pub fn desc(&self) -> &'static str {
                    match self {
                        $( $(
                            Self::$code_variant {..} => $code_desc,
                            // code_variant_match! {
                            //     Self::$code_variant, $code_desc,
                            //     $($($code_fields)*)?
                            // }
                        )* )?
                        $( $( Self::$sub_code_variant(val) => val.desc(), )* )?
                    }
                }
            }
            implem! {
                for $enum_name {
                    Display { |&self, fmt| self.desc().fmt(fmt) }
                }
            }

            $(
                impl $_enum_name {
                    $($impl_body)*
                }
            )?
        )+
    };
}

code_enums! {
    type Code = isize;
    unique name = error_codes;

    /// Param error codes.
    pub enum ParamErr {
        codes {
            CheckParamExpectConfigFilename = (3100, "[param] expects config filename"),
            CheckParamUsage = (3101, "[param] usage"),
            CheckParamMissingTlaModule = (3102, "[param] missing TLA module"),
            CheckParamNeedToSpecifyConfigDir = (3103, "[param] need to specify config dir"),
            CheckParamWorkerNumberRequired = (3104, "[param] worker number required"),
            CheckParamWorkerNumberTooSmall = (3105, "[param] worker number too small"),
            CheckParamWorkerNumberRequired2 = (3106, "[param] worker number required (2)"),
            CheckParamDepthRequired = (3107, "[param] depth required"),
            CheckParamDepthRequired2 = (3108, "[param] depth required (2)"),
            CheckParamTraceRequired = (3109, "[param] trace required"),
            CheckParamCoverageRequired = (3110, "[param] coverage required"),
            CheckParamCoverageRequired2 = (3111, "[param] coverage required"),
            CheckParamCoverageTooSmall = (3112, "[param] coverage too small"),
            CheckParamUnrecognized = (3113, "[param] unrecognized"),
            CheckParamTooManyInputFiles = (3114, "[param] too many input files"),
        }
    }

    /// Parser error codes.
    pub enum ParserErr {
        codes {
            SanyParserCheck1 = (4000, "[parser] check 1"),
            SanyParserCheck2 = (4001, "[parser] check 2"),
            SanyParserCheck3 = (4002, "[parser] check 3"),
        }
    }

    /// Weird error codes.
    pub enum WeirdErr {
        codes {
            Unknown = (-1, "[??] unknown"),
            UnitTest = (-123456, "[??] unit test"),
        }
    }

    /// Feature error codes.
    pub enum FeatureErr {
        codes {
            TlcFeatureUnsupported = (2156, "[feature] unsupported"),
            TlcFeatureUnsupportedLivenessSymmetry = (2279, "[feature] unsupported liveness symmetry"),
            TlcFeatureLivenessConstraints = (2284, "[feature] liveness constraints"),
        }
    }

    /// System error codes.
    pub enum SystemErr {
        codes {
            SystemOutOfMemory = (1001, "[system] out of memory"),
            SystemOutOfMemoryTooManyInit = (1002, "[system] out of memory (too many init)"),
            SystemOutOfMemoryLiveness = (1003, "[system] out of memory (liveness)"),
            SystemOutOfMemoryStackOverflow = (1005, "[system] out of memory (stack overflow)"),

            SystemErrorReadingPool = (2125, "[system] error reading pool"),
            SystemCheckpointRecoveryCorrupt = (2126, "[system] checkpoint recovery corrupt"),
            SystemErrorWritingPool = (2127, "[system] error writing pool"),
            SystemErrorCleaningPool = (2270, "[system] error cleaning pool"),
            SystemIndexError = (2134, "[system] index error"),
            SystemStreamEmtpy = (2135, "[system] stream empty"),
            SystemFileNull = (2137, "[system] file null"),
            SystemInterrupted = (2138, "[system] interrupted"),
            SystemUnableNotRenameFlie = (2160, "[system] unable not rename file"),
            SystemDiskIoErrorForFile = (2161, "[system] disk io error for file"),
            SystemMetadirExists = (2162, "[system] metadir exists"),
            SystemMetadirCreationError = (2163, "[system] metadir creation error"),
            SystemUnableToOpenFile = (2167, "[system] unable to open file"),

            SystemDiskgraphAccess = (2129, "[system] diskgraph access"),

            SystemErrorReadingStates = (2174, "[system] error reading states"),
            SystemErrorWritingStates = (2175, "[system] error writing states"),
        }
    }

    /// CLA error codes.
    pub enum ClaErr {
        codes {
            WrongCommandlineParamsSimulator = (1101, "[cla] params simulator"),
            WrongCommandlineParamsTlc = (1102, "[cla] params TLC"),
        }
    }

    /// Preprocessing error codes.
    pub enum PpErr {
        codes {
            TlcPpParsingValue = (2000, "[preproc] parsing value"),
            TlcPpFormattingValue = (2001, "[preproc] formatting value"),
        }
    }

    /// All TLC codes.
    pub enum TlcMsg {
        subs {
            Live(TlcLive),
            Distr(DistributedTlc),
            Msg(Tlc),
        }
    }

    /// Bad outcome in the analysis.
    pub enum TlcUnsafe {
        codes {
            TlcInvariantViolatedInitial = (2107, "[tlc] invariant violated initial"),
            TlcPropertyViolatedInitial = (2108, "[tlc] property violated initial"),
            TlcStateNotCompletelySpecifiedNext = (2109, "[tlc] tlc state not completely specified next"),
            TlcInvariantViolatedBehavior = (2110, "[tlc] invariant violated behavior") {
                invariant: String,
            } => |contents| {
                let line = contents.get_1_plain_str()?;
                let invariant = tlc::parse::invariant_violated_behavior(line)
                    .with_context(|| "failed to parse `invariant violated behavior`")?
                    .to_string();
                Ok(Self::TlcInvariantViolatedBehavior { invariant })
            },
            TlcInvariantViolatedLevel = (2146, "[tlc] invariant violated level"),
            TlcActionPropertyViolatedBehavior = (2112, "[tlc] action property violated behavior"),
            TlcDeadlockReached = (2114, "[tlc] deadlock reached"),

            TlcTemporalPropertyViolated = (2116, "[tlc] temporal property violated"),
        }
    }

    /// Problem in the analysis.
    pub enum TlcProblem {
        codes {
            TlcNoStatesSatisfyingInit = (2118, "[tlc] no states satisfying init"),
            TlcInvariantEvaluationFailed = (2111, "[tlc] invariant evaluation failed"),
            TlcActionPropertyEvaluationFailed = (2113, "[tlc] action property evaluation failed"),
            TlcValueAssertFailed = (2132, "[tlc] value assert failed") {
                failure_msg: Option<String>,
            } => |contents| {
                let (line_1, line_2) = contents.get_2_plain_str()?;
                tlc::parse::assertion_failure_1(line_1).context("parsing line 1 of assertion failure")?;
                let failure_msg = tlc::parse::assertion_failure_2(line_2).context("parsing line 2 of assertion failure")?;
                Ok(Self::TlcValueAssertFailed { failure_msg })
            },
        }
    }

    /// TLC error codes.
    pub enum TlcErr {
        codes {
            TlcMetadirExists = (2100, "[tlc] metadir exists"),
            TlcMetadirCanNotBeCreated = (2101, "[tlc] metadir cannot be created"),
            TlcInitialState = (2102, "[tlc] initial state"),
            TlcNestedExpression = (2103, "[tlc] nested expression") {
                locations: Vec<source::FileSpan>,
            } => |contents| {
                let mut lines = contents.only_plain_str_slices();
                {
                    let first = lines
                        .next()
                        .ok_or_else(|| anyhow!("expected at least three lines, got 0"))??;
                    tlc::parse::error_nested_expressions_1(first)
                        .with_context(|| anyhow!("parsing first line `{}`", first))?;
                    let second = lines
                        .next()
                        .ok_or_else(|| anyhow!("expected at least three lines, got 1"))??;
                    tlc::parse::error_nested_expressions_2(second)
                        .with_context(|| anyhow!("parsing second line `{}`", second))?;
                }
                let mut locations = Vec::with_capacity(17);

                for line in lines {
                    let line = line?;
                    let loc = tlc::parse::error_nested_expressions_location(line)
                        .with_context(|| anyhow!("parsing location `{}`", line))?;
                    locations.push(loc);
                }

                locations.shrink_to_fit();
                Ok(Self::TlcNestedExpression { locations })
            },
            TlcAssumptionFalse = (2104, "[tlc] assumption false"),
            TlcAssumptionEvaluationError = (2105, "[tlc] assumption evaluation error"),
            TlcStateNotCompletelySpecifiedInitial = (2106, "[tlc] state not completely specified initial"),

            TlcStatesAndNoNextAction = (2115, "[tlc] states and no next action"),
            TlcFailedToRecoverNext = (2117, "[tlc] failed to recover next"),
            TlcStringModuleNotFound = (2119, "[tlc] string module not found"),

            TlcErrorState = (2120, "[tlc] error state"),

            TlcStateNotCompletelySpecifiedLive = (2148, "[tlc] state not completely specified live"),

            TlcFailedToRecoverInit = (2123, "[tlc] failed to recover init"),
            TlcReporterDied = (2124, "[tlc] reporter died"),

            TlcBug = (2128, "[tlc] bug"),
            TlcFingerprintException = (2147, "[tlc] fingerprint exception"),

            TlcAaaaaaa = (2130, "[tlc] aaaaaaa"),
            TlcRegistryInitError = (2131, "[tlc] registry init error"),
            TlcChooseArgumentsWrong = (2164, "[tlc] choose arguments wrong"),
            TlcChooseUpperBound = (2165, "[tlc] choose upper bound"),

            TlcModuleValueJavaMethodOverride = (2154, "[tlc] module value java method override"),
            TlcModuleValueJavaMethodOverrideLoaded = (2168, "[tlc] module value java method override loaded"),
            TlcModuleValueJavaMethodOverrideMismatch = (2400, "[tlc] module value java method override mismatch"),
            TlcModuleValueJavaMethodOverrideModuleMismatch = (2402, "[tlc] module value java method override module mismatch"),
            TlcModuleValueJavaMethodOverrideIdentidierMismatch = (2403, "[tlc] module value java method override identifier mismatch"),
            TlcModuleOverrideStdout = (20000, "[tlc] module override stdout"),

            TlcFpNotInSet = (2133, "[tlc] fp not in set"),
            TlcFpValueAlreadyOnDisk = (2166, "[tlc] fp value already on disk"),

            TlcLiveBegraphFailedToConstruct = (2159, "[tlc] live begraph failed to construct"),
            TlcParameterMustBePostfix = (2136, "[tlc] parameter must be postfix"),
            TlcCouldNotDetermineSubscript = (2139, "[tlc] could not detemine subscript"),
            TlcSubscriptContainNoStateVar = (2140, "[tlc] subscript contain no state var"),
            TlcWrongTupleFieldName = (2141, "[tlc] wrong tuple field name"),
            TlcWrongRecordFieldName = (2142, "[tlc] wrong record field name"),
            TlcUnchangedVariableChanged = (2143, "[tlc] unchanged variable changed"),
            TlcExceptAppliedToUnknownField = (2144, "[tlc] except applied to unknown field"),

            TlcModuleTlcgetUndefined = (2145, "[tlc] module tlcget undefined"),
            TlcModuleCompareValue = (2155, "[tlc] module compare value"),
            TlcModuleCheckMemberOf = (2158, "[tlc] module check member of"),
            TlcModuleTransitiveClosure = (2157, "[tlc] module transitive closure"),
            TlcModuleArgumentError = (2169, "[tlc] module argument error"),
            TlcModuleArgumentErrorAn = (2266, "[tlc] module argument error an"),
            TlcModuleOneArgumentError = (2283, "[tlc] module one argument error"),
            TlcArgumentMismatch = (2170, "[tlc] argument mismatch"),
            TlcParsingFailed2 = (2171, "[tlc] parsing failed (2)"),
            TlcParsingFailed = (3002, "[tlc] parsing failed"),
            TlcTooManyPossibleStates = (2172, "[tlc] too many possible states"),
            TlcErrorReplacingModules = (2173, "[tlc] error replacing modules"),

            TlcModuleApplyingToWrongValue = (2176, "[tlc] module applying to wrong value"),
            TlcModuleBagUnion1 = (2177, "[tlc] module bag union 1"),
            TlcModuleOverflow = (2178, "[tlc] module overflow"),
            TlcModuleDivisionByZero = (2179, "[tlc] module division by zero"),
            TlcModuleNullPowerNull = (2180, "[tlc] module null power null"),
            TlcModuleComputingCardinality = (2181, "[tlc] module computing cardinality"),
            TlcModuleEvaluating = (2182, "[tlc] module evaluating"),
            TlcModuleArgumentNotInDomain = (2183, "[tlc] module argument not in domain"),
            TlcModuleApplyEmptySeq = (2184, "[tlc] module apply empty seq"),

            TlcSymmetrySetTooSmall = (2300, "[tlc] symmetry set too smal"),
            TlcSpecificationFeaturesTemporalQuantifier = (2301, "[tlc] specification features temporal quantifier"),

            TlcExpectedValue = (2215, "[tlc] expected value"),
            TlcExpectedExpression = (2246, "[tlc] expected expression"),
            TlcExpectedExpressionInComputing = (2247, "[tlc] expected expression in computing"),
            TlcExpectedExpressionInComputing2 = (2248, "[tlc] expected expression in computing (2)"),

            TlcEnabledWrongFormula = (2260, "[tlc] enabled wrong formula"),
            TlcEncounteredFormulaInPredicate = (2261, "[tlc] encountered formula in predicate"),

            TlcIntegerTooBig = (2265, "[tlc] integer too big"),
            TlcTraceTooLong = (2282, "[tlc] trace too long"),

            TlcTeSpecGenerationError = (2502, "[tlc msg] te spec generation error"),
        }
    }

    /// TLC countexample error codes.
    pub enum TlcCex {
        codes {
            TlcBackToState = (2122, "[tlc cex] back to state") {
                index: usize,
            } => |contents| {
                let mut lines = contents.only_plain_str_slices();
                let line = lines.next().ok_or_else(|| anyhow!("expected at least one line"))?
                    .with_context(|| "expected back-to-state line")?;
                let index = tlc::parse::back_to_state(line)
                    .with_context(|| anyhow!("failed to parse back-to-state content"))?;
                Ok(Self::TlcBackToState { index })
            },
            /// Originally called `TLC_STATE_PRINT3`.
            ///
            /// This error code accompanies the final state in a liveness property counter-example
            /// trace, when the trace ends in stuttering. For traces ending in a lasso, see
            /// [`Self::TlcBackToState`]. Note this code only accompanies the *final* state in the
            /// trace, and the prefix traces are accompanied by the [`Self::TlcTraceState`] code.
            TlcStuttering = (2218, "[tlc cex] stuttering"),
            /// Original documentation follows.
            ///
            /// This error code is used in the following situations:
            /// - during DFID model checking, when next state is not fully defined;
            /// - in the `tlc2.tool.CheckImpl` tool, when there is an invalid step;
            /// - during Simulation model checking, when maximum trace depth is reached.
            TlcStatePrint1 = (2216, "[tlc cex] state print (1)"),
            /// Originally called `TLC_STATE_PRINT2`.
            ///
            /// This error code is used in the following situations:
            /// - Printing a safety invariant violation error trace;
            /// - Printing every state except the final state of a liveness error trace;
            ///   the final state is printed with:
            ///   - [`Self::TlcBackToState`] for liveness traces ending in a lasso;
            ///   - [`Self::TlcStuttering`] for liveness traces ending in stuttering.
            TlcTraceState = (2217, "[tlc cex] trace state") {
                index: usize,
                state: cex::State,
            } => |contents| {
                let mut lines = contents.only_plain_str_slices();
                let state_info = lines.next().ok_or_else(|| anyhow!("expected at least one line"))?
                    .with_context(|| "expected state info line")?;
                let (index, state_info) = tlc::parse::state_info(state_info)
                    .with_context(|| anyhow!("failed to parse state info"))?;
                let mut state = cex::State::new(state_info);
                for id_value in lines {
                    let id_value = id_value.with_context(|| anyhow!("expected ident/value cex line"))?;
                    let (id, value) = tlc::parse::cex_ident_value(id_value)
                        .with_context(|| anyhow!("failed to parse ident/value pair"))?;
                    let _prev = state.insert(id.into(), value);
                    if let Some(_prev) = _prev {
                        bail!("TLC produced a cex state that mentions `{}` twice", id)
                    }
                }
                Ok(Self::TlcTraceState { index, state })
            },
        }
    }

    /// TLC live error codes.
    pub enum TlcLive {
        codes {
            TlcLiveImplied = (2212, "[tlc live] implied"),
            TlcLiveCannotHandleFormula = (2213, "[tlc live] cannot handle formula"),
            TlcLiveWrongFormulaFormat = (2214, "[tlc live] wrong formula format"),
            TlcLiveEncounteredActions = (2249, "[tlc live] encountered actions"),
            TlcLiveStatePredicateNonBool = (2250, "[tlc live] state predicate non bool"),
            TlcLiveCannotEvalFormula = (2251, "[tlc live] cannot eval formula"),
            TlcLiveEncounteredNonboolPredicate = (2252, "[tlc live] encountered nonbool predicate"),
            TlcLiveFormulaTautology = (2253, "[tlc live] formula tautology"),
        }
    }

    /// Distributed TLC error codes.
    pub enum DistributedTlc {
        codes {
            TlcDistributedServerRunning = (7000, "[distr tlc] server running"),
            TlcDistributedWorkerRegistered = (7001, "[distr tlc] worker registered"),
            TlcDistributedWorkerDeregistered = (7002, "[distr tlc] worker deregistered"),
            TlcDistributedWorkerStats = (7003, "[distr tlc] worker stats"),
            TlcDistributedServerNotRunning = (7004, "[distr tlc] server not running"),
            TlcDistributedVmVersion = (7005, "[distr tlc] vm version"),
            TlcDistributedWorkerLost = (7006, "[distr tlc] worker lost"),
            TlcDistributedExceedBlocksize = (7007, "[distr tlc] exceed blocksize"),
            TlcDistributedServerFpsetWaiting = (7008, "[distr tlc] server fpset waiting"),
            TlcDistributedServerFpsetRegistered = (7009, "[distr tlc] server fpset registered"),
            TlcDistributedServerFinished = (7010, "[distr tlc] server finished"),
        }
    }

    /// Normal TLC messages.
    pub enum Tlc {
        codes {
            TlcModeMc = (2187, "[tlc msg] mode mc"),
            TlcModeMcDfs = (2271, "[tlc msg] mode mc dfs"),
            TlcModeSimu = (2188, "[tlc msg] mode simu"),
            TlcComputingInitProgress = (2269, "[tlc msg] computing init progress"),
            TlcCheckingTemporalProps = (2192, "[tlc msg] checking temporal props"),
            TlcCheckingTemporalPropsEnd = (2267, "[tlc msg] checking temporal props end"),
            TlcSearchDepth = (2194, "[tlc msg] search depth") {
                depth: usize,
            } => |contents| {
                let line = contents.get_1_plain_str()?;
                tlc::parse::search_depth(line)
                    .with_context(|| anyhow!("failed to parse contents"))
            },
            TlcStateGraphOutdegree = (2268, "[tlc msg] state graph outdegree") {
                outdegree: usize,
                min: usize,
                max: usize,
                percentil_th: usize,
                percentil: usize,
            } => |contents| {
                let line = contents.get_1_plain_str()?;
                tlc::parse::graph_outdegree(line)
                    .with_context(|| anyhow!("failed to parse contents"))
            },
            TlcCheckpointStart = (2195, "[tlc msg] checkpoint start"),
            TlcCheckpointEnd = (2196, "[tlc msg] checkpoint end"),
            TlcCheckpointRecoverStart = (2197, "[tlc msg] checkpoint recover start"),
            TlcCheckpointRecoverEnd = (2198, "[tlc msg] checkpoint recover end"),
            TlcStats = (2199, "[tlc msg] stats") {
                generated: (Int, String),
                distinct: (Int, String),
                left: (Int, String),
            } => |contents| {
                let line = contents.get_1_plain_str()?;
                tlc::parse::stats(line)
                    .with_context(|| anyhow!("failed to parse contents"))
            },
            TlcStatsDfid = (2204, "[tlc msg] stats dfid"),
            TlcStatsSimu = (2210, "[tlc msg] stats simu"),
            TlcProgressStats = (2200, "[tlc msg] progress stats") {
                generated: (Int, String),
                gen_spm: Option<(Int, String)>,
                distinct: (Int, String),
                dist_spm: Option<(Int, String)>,
                left: (Int, String),
            } => |contents| {
                let line = contents.get_1_plain_str()?;
                tlc::parse::progress_stats(line)
                    .with_context(|| anyhow!("failed to parse contents"))
            },
            TlcCoverageStart = (2201, "[tlc msg] coverage start"),
            TlcCoverageEnd = (2202, "[tlc msg] coverage end"),
            TlcCheckpointRecoverEndDfid = (2203, "[tlc msg] checkpoint recover end dfid"),
            TlcProgressStartStatsDfid = (2205, "[tlc msg] progress start stats dfid"),
            TlcProgressStatsDfid = (2206, "[tlc msg] progress stats dfid"),
            TlcProgressSimu = (2209, "[tlc msg] progress simu"),
            TlcFpCompleted = (2211, "[tlc msg] fp completed"),

            TlcCoverageMismatch = (2776, "[tlc msg] coverage mismatch"),
            TlcCoverageValue = (2221, "[tlc msg] coverage value"),
            TlcCoverageValueCost = (2775, "[tlc msg] coverage value cost"),
            TlcCoverageNext = (2772, "[tlc msg] coverage next"),
            TlcCoverageInit = (2773, "[tlc msg] coverage init"),
            TlcCoverageProperty = (2774, "[tlc msg] coverage property"),
            TlcCoverageConstraint = (2778, "[tlc msg] coverage constraint"),
            TlcCoverageEndOverhead = (2777, "[tlc msg] coverage end overhead"),

            TlcVersion = (2262, "[tlc msg] version"),

            TlcEnvironmentJvmGc = (2401, "[tlc msg] environment jvm gc"),

            TlcTeSpecGenerationComplete = (2501, "[tlc msg] te spec generation complete"),
        }
    }

    /// CFG error codes.
    pub enum CfgErr {
        codes {
            CfgErrorReadingFile = (5001, "[cfg] error reading file"),
            CfgGeneral = (5002, "[cfg] general"),
            CfgMissingId = (5003, "[cfg] missing id"),
            CfgTwiceKeyword = (5004, "[cfg] twice keyword"),
            CfgExpectId = (5005, "[cfg] expect id"),
            CfgExpectedSymbol = (5006, "[cfg] expected symbol"),

            TlcConfigValueNotAssignedToConstantParam = (2222, "[cfg] config value not assigned to constant param"),
            TlcConfigRhsIdAppearedAfterLhsId = (2223, "[cfg] config rhs id appeared after lhs id"),
            TlcConfigWrongSubstitution = (2224, "[cfg] config wrong substitution"),
            TlcConfigWrongSubstitutionNumberOfArgs = (2225, "[cfg] config wrong substitution number of args"),
            TlcConfigUndefinedOrNoOperator = (2280, "[cfg] config undefined or no operator"),
            TlcConfigSubstitutionNonConstant = (2281, "[cfg] config substitution non constant"),
            TlcConfigIdDoesNotAppearInSpec = (2226, "[cfg] config id does not appear in spec"),
            TlcConfigNotBothSpecAndInit = (2227, "[cfg] config not both spec and init"),
            TlcConfigIdRequiresNoArg = (2228, "[cfg] config id requires no arg"),
            TlcConfigSpecifiedNotDefined = (2229, "[cfg] config specified not defined"),
            TlcConfigIdHasValue = (2230, "[cfg] config id has value"),
            TlcConfigMissingInit = (2231, "[cfg] config missing init"),
            TlcConfigMissingNext = (2232, "[cfg] config missing next"),
            TlcConfiIdMustNotBeConstant = (2233, "[cfg] config id must not be constant"),
            TlcConfigOpNoArgs = (2234, "[cfg] config op no args"),
            TlcConfigOpNotInSpec = (2235, "[cfg] config op not in spec"),
            TlcConfigOpIsEqual = (2236, "[cfg] config op is equal"),
            TlcConfigSpecIsTrivial = (2237, "[cfg] config spec is trivial"),
            TlcCantHandleSubscript = (2238, "[cfg] can't handle subscript"),
            TlcCantHandleConjunct = (2239, "[cfg] can't handle conjunct"),
            TlcCantHandleTooManyNextStateRels = (2240, "[cfg] can't handle too many next state rels"),
            TlcConfigPropertyNotCorrectlyDefined = (2241, "[cfg] config property not correctly defined"),
            TlcConfigOpArityInconsistent = (2242, "[cfg] config op arity inconsistent"),
            TlcConfigNoStateType = (2243, "[cfg] config no state type"),
            TlcCantHandleRealNumbers = (2244, "[cfg] can't handle real numbers"),
            TlcNoModules = (2245, "[cfg] no modules"),
        }
    }

    /// Error codes.
    pub enum Err {
        codes {
            CheckFailedToCheck = (3000, "failed to check"),
            CheckCouldNotReadTrace = (3001, "could not read trace"),
            /// Custom error, triggered when no java runtime is found.
            ///
            /// Code is `1_000_000` to avoid clashes, though we **do check** for clashes.
            ///
            /// Note that there are custom [`TopMsg::parse_start`] and [`TopMsg::parse_end`]
            /// behavior for this error since it's not a real TLC-level error, and thus cannot be
            /// parsed with TLC message code.
            NoJavaRuntime = (1_000_000, "unable to locate java runtime"),
        }
        subs {
            Param(ParamErr),
            Parser(ParserErr),
            Weird(WeirdErr),
            Feature(FeatureErr),
            System(SystemErr),
            Cla(ClaErr),
            Pp(PpErr),
            Tlc(TlcErr),
            Cfg(CfgErr),
            Problem(TlcProblem),
        }
    }

    /// Statuses.
    pub enum Status {
        codes {
            TlcStarting = (2185, "[status] starting"),
            TlcFinished = (2186, "[status] finished") {
                runtime: time::Duration,
            } => |contents| {
                let line = contents.get_1_plain_str()?;
                tlc::parse::finished(line)
                    .with_context(|| anyhow!("failed to parse contents"))
            },

            TlcSanyStart = (2220, "[status] sany start"),
            TlcSanyEnd = (2219, "[status] sany end"),

            TlcComputingInit = (2189, "[status] computing init"),

            TlcInitGenerated1 = (2190, "[status] init generated (1)") {
                state_count: (Int, String),
                end_time: chrono::NaiveDateTime,
            } => |contents| {
                let line = contents.get_1_plain_str()?;
                tlc::parse::init_generated_1(line)
                    .with_context(|| anyhow!("failed to parse contents"))
            },
            TlcInitGenerated2 = (2191, "[status] init generated (2)"),
            TlcInitGenerated3 = (2207, "[status] init generated (3)"),
            TlcInitGenerated4 = (2208, "[status] init generated (4)"),

            TlcBehaviorUpToThisPoint = (2121, "[tlc] behavior up to this point"),

            TlcCounterExample = (2264, "[tlc msg] counter example"),

            TlcSuccess = (2193, "[tlc msg] success"),
        }
    }

    /// Message codes.
    pub enum Msg {
        codes {
            General = (1000, "general"),
        }
        subs {
            Tlc(TlcMsg),
            Status(Status),
            Cex(TlcCex),
            TlcUnsafe(TlcUnsafe),
        }
    }

    /// Top-level codes, root of the error code tree.
    pub enum TopMsg {
        subs {
            /// Error codes.
            Err(Err),
            /// Message codes.
            Msg(Msg),
        }
    }
}

implem! {
    impl(T: Into<Err>) for TopMsg {
        From<T> { |t| Self::Err(t.into()) }
    }
    for TopMsg {
        From<Msg> { |t| Self::Msg(t.into()) }
    }
}

lazy_static! {
    /// # TODO
    ///
    /// - this regex starts with `[:]?\s*` to account for problems in TLC's formatting. See
    ///     https://github.com/tlaplus/tlaplus/issues/732 for more details. Remove when the issue is
    ///     fixed.
    static ref START_REGEX: Regex =
        Regex::new(r"^[:]?\s*@!@!@STARTMSG\s+(-?\d+):(\d+)\s+@!@!@$").unwrap();
    static ref END_REGEX: Regex = Regex::new(r"^@!@!@ENDMSG\s+(-?\d+)\s+@!@!@$").unwrap();
}
impl TopMsg {
    /// Constructor.
    pub fn new(code: Code, contents: &tlc::msg::Elms) -> Res<Self> {
        Self::from_code(code, contents)?
            .ok_or_else(|| anyhow!("unknown TLC message code `{}`", code))
    }
    /// Turns itself into a result.
    pub fn into_res(&self, msg: &tlc::msg::Msg) -> Res<&Msg> {
        match self {
            Self::Msg(msg) => Ok(msg),
            Self::Err(e) => {
                let lines: Option<String> = msg.lines().into_iter().fold(None, |mut acc, line| {
                    if let Some(string) = &mut acc {
                        string.push_str("\n");
                        string.push_str(line);
                        acc
                    } else {
                        Some(line.to_string())
                    }
                });
                let top_most = format!("TLC-level error: {}", e);
                let e = if let Some(lines) = lines {
                    Error::msg(lines).context(top_most)
                } else {
                    Error::msg(top_most)
                };
                Err(e)
            }
        }
    }

    /// Turns itself into an error.
    pub fn unwrap_err(&self) -> &Err {
        match self {
            Self::Msg(_) => panic!("cannot `TopMsg::unwrap_err` on a normal message"),
            Self::Err(e) => e,
        }
    }

    /// True if the message is an error.
    pub fn is_err(&self) -> bool {
        match self {
            Self::Msg(_) => false,
            Self::Err(_) => true,
        }
    }

    /// Status message.
    pub fn status(&self) -> Option<&Status> {
        match self {
            Self::Msg(Msg::Status(status)) => Some(status),
            _ => None,
        }
    }

    /// True if the code is for general messages.
    pub fn is_general(&self) -> bool {
        match self {
            TopMsg::Msg(Msg::General) => true,
            _ => false,
        }
    }

    /// Parses a message start.
    pub fn parse_start(s: impl AsRef<str>) -> Res<Option<(Code, usize)>> {
        let s = s.as_ref();

        if s.contains("Unable to locate a Java Runtime") {
            return Ok(Some((Err::NoJavaRuntime.code(), 0)));
        }

        let mut captures = START_REGEX.captures_iter(s);

        let capture = if let Some(c) = captures.next() {
            c
        } else {
            return Ok(None);
        };
        if captures.next().is_some() {
            bail!(msg::fatal!("START_REGEX has more than one capture"))
        }

        // Expecting an array containing
        // - the full match
        // - code group
        // - trail `usize` group
        const EXP_LEN: usize = 3;
        if capture.len() != EXP_LEN {
            bail!(msg::fatal!(
                "START_REGEX's capture has {} element(s), expected {}",
                EXP_LEN,
                capture.len(),
            ))
        }

        let code = {
            let code = &capture[1];
            let code = isize::from_str_radix(code, 10)
                .with_context(|| msg::fatal!("failed to parse `isize` value `{}`", code))?;
            Code::new(code)
            // Self::from_code(code).ok_or_else(|| anyhow!("unknown TLC *error code* `{}`", code))?
        };
        let trail = {
            let trail = &capture[2];
            usize::from_str_radix(trail, 10)
                .with_context(|| msg::fatal!("failed to parse `usize` value `{}`", trail))?
        };
        Ok(Some((code, trail)))
    }

    /// Parses a message end.
    pub fn parse_end(s: impl AsRef<str>) -> Res<Option<Code>> {
        let s = s.as_ref();

        if s.contains("on installing Java") {
            return Ok(Some(Err::NoJavaRuntime.code()));
        }

        let mut captures = END_REGEX.captures_iter(s);

        let capture = if let Some(c) = captures.next() {
            c
        } else {
            return Ok(None);
        };
        if captures.next().is_some() {
            bail!(msg::fatal!("END_REGEX has more than one capture"))
        }

        // Expecting an array containing
        // - the full match
        // - code group
        // - trail `usize` group
        const EXP_LEN: usize = 2;
        if capture.len() != EXP_LEN {
            bail!(msg::fatal!(
                "END_REGEX's capture has {} element(s), expected {}",
                EXP_LEN,
                capture.len(),
            ))
        }

        let code = {
            let code = &capture[1];
            let code = isize::from_str_radix(code, 10)
                .with_context(|| msg::fatal!("failed to parse `isize` value `{}`", code))?;
            Code::new(code)
            // Self::from_code(code).ok_or_else(|| anyhow!("unknown TLC *error code* `{}`", code))?
        };
        Ok(Some(code))
    }

    /// Returns a reference to the internal error if any.
    pub fn as_err(&self) -> Option<&Err> {
        match self {
            Self::Msg(_) => None,
            Self::Err(e) => Some(e),
        }
    }
}

macro_rules! unhandled_error {
    ($styles:expr, $err:expr) => {{
        // The URL for the github issue was written with
        //
        // https://github.com/sindresorhus/new-github-release-url
        //
        // which is pretty awesome despite being written in JS.
        println!("\
Hi there early {matla} user 💌

As you (should) know, {matla} is still very young. In particular, TLC-level error-handling is quite
immature. Improving it requires having concrete examples for each of TLC's many different errors.

It seems you just triggered an error for which we do have a handler yet. We would greatly appreciate
if you could take a few minutes to report this as an issue on our github. First, open this link to
check if this error has already been reported:

    https://github.com/OCamlPro/matla/issues?q=is%3Aissue+is%3Aopen+report+on+{err}

If this search returns a result, consider adding a `+1` or something to let us know how to
prioritize error kinds. If the search returns nothing, you can use the following link to create an
issue with a prepared title and body tailored to this specific error.

    https://github.com/OCamlPro/matla/issues/new?body=%E2%9A%A0+if+you+are+patient+enough%2C+try+to+add+a+TLA%2B+example+triggering+the+error%2C+or+at+least+some+description+of+how+you+triggered+it%3B+thank+you%21+%E2%9A%A0&title=Report+on+TLC-level+error+{err}

Doing so would help us {a_great_deal}!

Thank you for using {matla} and have a great day 💖\n\
        ",
            err = format!("{:?}", $err),
            a_great_deal = $styles.bold.paint("a great deal"),
            matla = $styles.good.paint("matla"),
        )
    }};
}

impl Err {
    /// Reports itself.
    pub fn report(&self) {
        let styles = conf::Styles::new();
        match self {
            Self::Problem(inner) => inner.report(&styles),
            _ => unhandled_error!(styles, self.code().get()),
        }
    }

    /// Turns itself into a real TLC error.
    pub fn into_tlc_error(self, subs: tlc::msg::Elms) -> Res<tlc::TlcError> {
        // println!("self: {}", self);
        // println!("subs:");
        // for line in subs.clone().into_string()?.lines() {
        //     println!("  {}", line)
        // }
        match self {
            Self::CheckFailedToCheck | Self::CheckCouldNotReadTrace => {
                Ok(tlc::err::SemanticError::new(
                    ModuleOrTop::TopTla,
                    Some(self.into()),
                    subs.into_string()?,
                    None,
                )
                .into())
            }
            Self::NoJavaRuntime => Ok(tlc::TlcError::NoJavaRuntime),
            Self::Param(e) => e.into_tlc_error(subs),
            Self::Parser(e) => e.into_tlc_error(subs),
            Self::Weird(e) => e.into_tlc_error(subs),
            Self::Feature(e) => e.into_tlc_error(subs),
            Self::System(e) => e.into_tlc_error(subs),
            Self::Cla(e) => e.into_tlc_error(subs),
            Self::Pp(e) => e.into_tlc_error(subs),
            Self::Tlc(e) => e.into_tlc_error(subs),
            Self::Cfg(e) => e.into_tlc_error(subs),
            Self::Problem(e) => e.into_tlc_error(subs),
        }
    }
}

impl PpErr {
    /// Turns itself into a real TLC error.
    pub fn into_tlc_error(self, subs: tlc::msg::Elms) -> Res<tlc::TlcError> {
        // use TlcErr::*;
        let blah = match self {
            _ => subs.into_string()?,
        };
        Ok(
            tlc::err::SemanticError::new(ModuleOrTop::TopTla, Some(Err::Pp(self)), blah, None)
                .into(),
        )
    }
}

impl ClaErr {
    /// Turns itself into a real TLC error.
    pub fn into_tlc_error(self, subs: tlc::msg::Elms) -> Res<tlc::TlcError> {
        // use TlcErr::*;
        let blah = match self {
            _ => subs.into_string()?,
        };
        Ok(
            tlc::err::SemanticError::new(ModuleOrTop::TopTla, Some(Err::Cla(self)), blah, None)
                .into(),
        )
    }
}

impl SystemErr {
    /// Turns itself into a real TLC error.
    pub fn into_tlc_error(self, subs: tlc::msg::Elms) -> Res<tlc::TlcError> {
        // use TlcErr::*;
        let blah = match self {
            _ => subs.into_string()?,
        };
        Ok(
            tlc::err::SemanticError::new(ModuleOrTop::TopTla, Some(Err::System(self)), blah, None)
                .into(),
        )
    }
}

impl FeatureErr {
    /// Turns itself into a real TLC error.
    pub fn into_tlc_error(self, subs: tlc::msg::Elms) -> Res<tlc::TlcError> {
        // use TlcErr::*;
        let blah = match self {
            _ => subs.into_string()?,
        };
        Ok(
            tlc::err::SemanticError::new(ModuleOrTop::TopTla, Some(Err::Feature(self)), blah, None)
                .into(),
        )
    }
}

impl WeirdErr {
    /// Turns itself into a real TLC error.
    pub fn into_tlc_error(self, subs: tlc::msg::Elms) -> Res<tlc::TlcError> {
        // use TlcErr::*;
        let blah = match self {
            _ => subs.into_string()?,
        };
        Ok(
            tlc::err::SemanticError::new(ModuleOrTop::TopTla, Some(Err::Weird(self)), blah, None)
                .into(),
        )
    }
}

impl ParserErr {
    /// Turns itself into a real TLC error.
    pub fn into_tlc_error(self, subs: tlc::msg::Elms) -> Res<tlc::TlcError> {
        // use TlcErr::*;
        let blah = match self {
            _ => subs.into_string()?,
        };
        Ok(
            tlc::err::SemanticError::new(ModuleOrTop::TopTla, Some(Err::Parser(self)), blah, None)
                .into(),
        )
    }
}

impl ParamErr {
    /// Turns itself into a real TLC error.
    pub fn into_tlc_error(self, subs: tlc::msg::Elms) -> Res<tlc::TlcError> {
        // use TlcErr::*;
        let blah = match self {
            _ => subs.into_string()?,
        };
        Ok(
            tlc::err::SemanticError::new(ModuleOrTop::TopTla, Some(Err::Param(self)), blah, None)
                .into(),
        )
    }
}

impl TlcErr {
    /// Turns itself into a real TLC error.
    pub fn into_tlc_error(self, subs: tlc::msg::Elms) -> Res<tlc::TlcError> {
        use TlcErr::*;
        let blah = match self {
            TlcParsingFailed2 => {
                format!(
                    "{}\n\n\
                    This error often appears when your module header \
                    and/or footer are ill-formed.\n\
                    Make sure your module starts with\n    \
                    ---- MODULE <file_basename> ----\n\
                    and ends with\n    \
                    ====\n\
                    where `<file_basename>` is your file's name without `.tla`.\
                    ",
                    subs.into_string()?
                )
            }
            _ => subs.into_string()?,
        };
        Ok(
            tlc::err::SemanticError::new(ModuleOrTop::TopTla, Some(Err::Tlc(self)), blah, None)
                .into(),
        )
    }
}

impl CfgErr {
    /// Turns itself into a real TLC error.
    pub fn into_tlc_error(self, subs: tlc::msg::Elms) -> Res<tlc::TlcError> {
        use CfgErr::*;
        let blah = match self {
            TlcConfigMissingInit => {
                "The `.cfg` file provided does not specify an initial state predicate.\n\
                Modules imported with a parameterized `INSTANCE` statement \
                can also cause this error."
                    .into()
            }
            _ => subs.into_string()?,
        };
        Ok(
            tlc::err::SemanticError::new(ModuleOrTop::TopCfg, Some(Err::Cfg(self)), blah, None)
                .into(),
        )
    }
}

impl TlcProblem {
    /// Reports itself.
    pub fn report(&self, styles: &conf::Styles) {
        match self {
            Self::TlcValueAssertFailed { failure_msg } => {
                print!(
                    "state exploration triggered an {}",
                    styles.fatal.paint("assertion failure")
                );
                if let Some(msg) = failure_msg {
                    if !msg.is_empty() {
                        println!(" with a message:");
                        for line in msg.lines() {
                            println!("> {}", styles.bad.paint(line));
                        }
                        return ();
                    }
                }
                println!(" with no failure message");
            }
            _ => unhandled_error!(styles, self.code().get()),
        }
    }

    /// Turns itself into a real TLC error.
    pub fn into_tlc_error(self, _subs: tlc::msg::Elms) -> Res<tlc::TlcError> {
        match self {
            Self::TlcValueAssertFailed { failure_msg } => {
                let kind = tlc::err::RunErrorKind::AssertFailed { msg: failure_msg };
                Ok(tlc::TlcError::new_run(tlc::err::RunError::new(kind)))
            }
            _ => todo!(),
        }
    }
}

impl Msg {
    /// True if the code is for general messages.
    pub fn is_general(&self) -> bool {
        match self {
            Self::General => true,
            _ => false,
        }
    }
}

code_enums! {
    type Code = i32;
    unique name = exit_codes;

    /// Enumeration of the outcomes of a TLC run.
    pub enum Exit {
        codes {
            PlainError = (255, "error"),
            Success = (0, "success"),
        }
        subs {
            Violation(ExitViolation),
            Failure(ExitFailure),
            Error(ExitError),
        }
    }

    /// Violations.
    pub enum ExitViolation {
        codes {
            ViolationAssumption = (10, "[violation] assumption"),
            ViolationDeadlock = (11, "[violation] deadlock"),
            ViolationSafety = (12, "[violation] safety"),
            ViolationLiveness = (13, "[violation] liveness"),
            ViolationAssert = (14, "[violation] assert"),
        }
    }

    /// Failures.
    pub enum ExitFailure {
        codes {
            FailureSpecEval = (75, "[failure] spec eval"),
            FailureSafetyEval = (76, "[failure] safety eval"),
            FailureLivenessEval = (77, "[failure] liveness eval"),
        }
    }

    /// Errors.
    pub enum ExitError {
        codes {
            ErrorSpecParse = (150, "[error] spec parse"),
            ErrorConfigParse = (151, "[error] config parse"),
            ErrorStatespaceTooLarge = (152, "[error] statespace too large"),
            ErrorSystem = (153, "[error] system"),
        }
    }
}

impl Exit {
    /// Yields true if the exit code corresponds to an actual error.
    ///
    /// In other words, returns false on *safe* or *unsafe* results.
    pub fn is_error(&self) -> bool {
        match self {
            Self::Success
            | Self::Violation(ExitViolation::ViolationSafety | ExitViolation::ViolationLiveness) => {
                false
            }
            Self::Violation(
                ExitViolation::ViolationAssumption
                | ExitViolation::ViolationDeadlock
                | ExitViolation::ViolationAssert,
            )
            | Self::PlainError
            | Self::Error(_)
            | Self::Failure(_) => true,
        }
    }
}
