(defn- neorg/export/linkable-html-impl
  [ctx target node]
  # TODO: accept ctx... we should pass it to inline tags
  # (def ctx @{})
  (def href (_neorg/export/linkable-href (ctx :path) target))
  (def attrs {:href href})
  (def markup (node :markup))
  (string
    "<a"
    (html/create-attrs
      (html/merge-attrs
        attrs
        (filter-attrs :html (parse-attrs (node :attrs)))))
    ">"
    (if markup
      (string/join (map |(norg/export/inline :html $ ctx) markup))
      (html/escape (attrs :href)))
    "</a>"))

(defn neorg/export/linkable
  "Export resolved linkable into export language"
  [ctx lang target node]
  (case lang
    :html (neorg/export/linkable-html-impl ctx target node)
    (error "only HTML is supported")))

# (defn neorg/resolve-anchor
#   "Resolve anchor reference from current document. Using NorgBerg cache if possible"
#   [ctx node]
#   (error "todo"))
