(def- iso8601/grammar
  (peg/compile
    '{:year   (<- (repeat 4 :d))
      :month  (<- (+ (* "0" (range "19"))
                     (* "1" (range "02"))))
      :day    (<- (+ (* "0" (range "19"))
                     (* (range "12") :d)
                     (* "3" (range "01"))))
      :hour   (<- (+ (* (range "12") :d)
                     (* "2" (range "03"))))
      :minute (<- (* (range "05") :d))
      :second (<- (* (range "05") :d))
      :tzsign (<- (+ "+" "-"))
      :tzhour (<- (* (range "01") :d))
      :tzsep  ":"
      :tzmin  (<- (* (range "05") :d))
      :date   (* :year "-" :month "-" :day)
      :time   (* :hour ":" :minute ":" :second)
      :tz     (* :tzsign :tzhour :tzsep :tzmin)
      :main   (* :date "T" :time :tz)}))

(defn iso8601/parse [ts]
  (var parts (peg/match iso8601/grammar ts))
  (if-not parts
    (error "invalid timestamp format"))
  (let [[ys ms ds hs mins ss tzn tzh tzm] parts]
    (var year (scan-number ys))
    (var month (dec (scan-number ms)))
    (var day  (dec (scan-number ds)))
    (var hour (scan-number hs))
    (var min  (scan-number mins))
    (var sec  (scan-number ss))
    (var sign (if (= tzn "+") 1 -1))
    (var tz-h (scan-number tzh))
    (var tz-m (scan-number tzm))
    (var tz-offset (* sign (+ (* tz-h 3600) (* tz-m 60))))
    (var dt {:year year
             :month month
             :month-day day
             :hours hour
             :minutes min
             :seconds sec})
    (- (os/mktime dt) tz-offset)))

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
       (def docs (filter |((neorg/doc/read-meta $) "created")
                         docs))
       (def docs (sorted-by
                   |(* -1 (iso8601/parse ((neorg/doc/read-meta $) "created")))
                   docs))
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
