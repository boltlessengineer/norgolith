(defn- neorg/export/linkable-html-impl
  [ctx target node]
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

(defn neorg/doc/read-meta
  "get metadata from give path.
   metadata can only be defined from very first element of entire document"
  [path]
  (def neorg/parse-file ((compile 'neorg/parse-file)))
  (def ast (neorg/parse-file path))
  (def ctx @{:meta @{}})
  (norg/export/block :meta ((ast :blocks) 0) ctx)
  (ctx :meta))

# (defn neorg/resolve-anchor
#   "Resolve anchor reference from current document. Using NorgBerg cache if possible"
#   [ctx node]
#   (error "todo"))

(put norg/ast/tag
     "ul-glob-docs"
     (fn [ctx [query]]
       (def docs (neorg/glob-docs (ctx :path) query))
       (def items (seq [path :in docs
                        :let [meta (neorg/doc/read-meta path)]
                        :unless (meta "draft")]
                    (def target [:app (neorg/create-app-target (ctx :path) path)])
                    (def title (meta "title"))
                    (def markup [{:kind :text
                                  :text title}])
                    (def link {:kind :link
                               :target target
                               :markup markup})
                    (def paragraph {:kind :paragraph
                                    :inlines [link]})
                    {:kind :list-item
                     :contents [paragraph]}))
       [{:kind :unordered-list
         :items items}]))

(put norg/ast/tag
     "private"
     (fn [ctx _args _lines]
       []))

(put norg/ast/tag
     "todo"
     (fn [ctx _args]
       (print "todo")
       []))

(put norg/ast/tag
     "\\todo"
     (fn [ctx _args _markup]
       (print "inline-todo")
       []))
