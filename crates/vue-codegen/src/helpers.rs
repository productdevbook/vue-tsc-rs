//! Helper types and utilities for code generation.

/// Vue-tsc helper types that are injected into the generated code.
pub const VLS_HELPER_TYPES: &str = r#"
// Helper types for Vue type checking

type __VLS_Prettify<T> = { [K in keyof T]: T[K] } & {};

type __VLS_WithDefaults<P, D> = {
    [K in keyof P]: K extends keyof D
        ? P[K] extends undefined
            ? D[K]
            : P[K]
        : P[K];
};

type __VLS_NonUndefinedable<T> = T extends undefined ? never : T;

type __VLS_TypePropsToOption<T> = {
    [K in keyof T]-?: {} extends Pick<T, K>
        ? { type: __VLS_PropType<__VLS_NonUndefinedable<T[K]>>; required?: false }
        : { type: __VLS_PropType<T[K]>; required: true };
};

type __VLS_WithComponent<N, C> = C;

type __VLS_IntrinsicElements = {
    [K in keyof HTMLElementTagNameMap]: Partial<HTMLElementTagNameMap[K]>;
} & {
    [K in keyof SVGElementTagNameMap]: Partial<SVGElementTagNameMap[K]>;
};

interface __VLS_TemplateContext {
    $slots: any;
    $attrs: any;
    $refs: any;
    $el: any;
    $emit: any;
    $props: any;
}

declare function __VLS_asFunctionalComponent<T>(
    t: T,
): T extends new (...args: any[]) => any
    ? InstanceType<T> extends { $props: infer P }
        ? (props: P & Record<string, unknown>) => any
        : never
    : T;

declare function __VLS_getVForSourceType<T>(
    source: T,
): T extends number
    ? number[]
    : T extends string
    ? string[]
    : T extends readonly (infer U)[]
    ? U[]
    : T extends Iterable<infer U>
    ? U[]
    : { [K in keyof T]: T[K] }[];

declare function __VLS_getSlotParams<T>(
    slot: T,
): T extends (...args: any[]) => any ? Parameters<T>[0] : never;

declare function __VLS_elementAsFunction<T extends keyof __VLS_IntrinsicElements>(
    tag: T,
): (props: __VLS_IntrinsicElements[T]) => void;

declare function __VLS_componentAsFunction<T>(
    component: T,
): T extends new (...args: any[]) => infer R
    ? (props: R extends { $props: infer P } ? P : never) => void
    : T extends (...args: any[]) => any
    ? T
    : never;

declare function __VLS_resolveComponent<T extends string>(
    name: T,
): any;

declare function __VLS_resolveDirective<T extends string>(
    name: T,
): any;

declare function __VLS_withAsyncContext<T>(
    getAwaitable: () => Promise<T>,
): Promise<T>;
"#;

/// Names used in generated code.
pub mod names {
    pub const PROPS: &str = "__VLS_props";
    pub const EMIT: &str = "__VLS_emit";
    pub const SLOTS: &str = "__VLS_slots";
    pub const CTX: &str = "__VLS_ctx";
    pub const REFS: &str = "__VLS_refs";
    pub const SETUP: &str = "__VLS_setup";
    pub const COMPONENT: &str = "__VLS_component";
    pub const TEMPLATE: &str = "__VLS_template";
}

/// Built-in Vue global components.
pub const BUILTIN_COMPONENTS: &[&str] = &[
    "Transition",
    "TransitionGroup",
    "KeepAlive",
    "Suspense",
    "Teleport",
];

/// Built-in Vue directives.
pub const BUILTIN_DIRECTIVES: &[&str] = &[
    "v-if",
    "v-else",
    "v-else-if",
    "v-for",
    "v-show",
    "v-bind",
    "v-on",
    "v-model",
    "v-slot",
    "v-pre",
    "v-cloak",
    "v-once",
    "v-memo",
    "v-html",
    "v-text",
];

/// Check if a component name is a built-in.
pub fn is_builtin_component(name: &str) -> bool {
    BUILTIN_COMPONENTS
        .iter()
        .any(|&builtin| builtin.eq_ignore_ascii_case(name))
}

/// Check if a tag is an HTML element.
pub fn is_html_tag(tag: &str) -> bool {
    HTML_TAGS.contains(&tag.to_lowercase().as_str())
}

/// Check if a tag is an SVG element.
pub fn is_svg_tag(tag: &str) -> bool {
    SVG_TAGS.contains(&tag.to_lowercase().as_str())
}

/// HTML tags.
const HTML_TAGS: &[&str] = &[
    "a", "abbr", "address", "area", "article", "aside", "audio", "b", "base", "bdi", "bdo",
    "blockquote", "body", "br", "button", "canvas", "caption", "cite", "code", "col", "colgroup",
    "data", "datalist", "dd", "del", "details", "dfn", "dialog", "div", "dl", "dt", "em", "embed",
    "fieldset", "figcaption", "figure", "footer", "form", "h1", "h2", "h3", "h4", "h5", "h6",
    "head", "header", "hgroup", "hr", "html", "i", "iframe", "img", "input", "ins", "kbd", "label",
    "legend", "li", "link", "main", "map", "mark", "math", "menu", "meta", "meter", "nav",
    "noscript", "object", "ol", "optgroup", "option", "output", "p", "param", "picture", "pre",
    "progress", "q", "rp", "rt", "ruby", "s", "samp", "script", "search", "section", "select",
    "slot", "small", "source", "span", "strong", "style", "sub", "summary", "sup", "svg", "table",
    "tbody", "td", "template", "textarea", "tfoot", "th", "thead", "time", "title", "tr", "track",
    "u", "ul", "var", "video", "wbr",
];

/// SVG tags.
const SVG_TAGS: &[&str] = &[
    "svg", "animate", "animateMotion", "animateTransform", "circle", "clipPath", "defs", "desc",
    "ellipse", "feBlend", "feColorMatrix", "feComponentTransfer", "feComposite",
    "feConvolveMatrix", "feDiffuseLighting", "feDisplacementMap", "feDistantLight", "feDropShadow",
    "feFlood", "feFuncA", "feFuncB", "feFuncG", "feFuncR", "feGaussianBlur", "feImage", "feMerge",
    "feMergeNode", "feMorphology", "feOffset", "fePointLight", "feSpecularLighting", "feSpotLight",
    "feTile", "feTurbulence", "filter", "foreignObject", "g", "image", "line", "linearGradient",
    "marker", "mask", "metadata", "mpath", "path", "pattern", "polygon", "polyline",
    "radialGradient", "rect", "set", "stop", "switch", "symbol", "text", "textPath", "tspan",
    "use", "view",
];
